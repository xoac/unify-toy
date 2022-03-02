use crate::{hotels, input, serde::checkin_checkout_format};
use anyhow::Context;
use csv_async::AsyncWriterBuilder;
use futures::StreamExt;
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};
use std::path::Path;
use time::Date;
use tokio::{
    fs,
    io::{AsyncWrite, BufWriter},
};
use tokio_stream::Stream;

use crate::city_code::CityCode;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RoomTypeWithMeal {
    room_type: String,
    meal: String,
}

impl RoomTypeWithMeal {
    fn as_string(&self) -> String {
        format!("{} {}", self.room_type, self.meal)
    }

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Output {
    #[serde(
        rename = "room_type meal",
        serialize_with = "RoomTypeWithMeal::serialize"
    )]
    room_type_and_meal: RoomTypeWithMeal, // - room code and meal code, space seprated
    room_code: String,   // - room code
    source: String,      // - name of the price source
    hotel_name: String,  // - name of the hotel
    city_name: String,   // - name of the city
    city_code: CityCode, // - 3-letters city code
    hotel_category: f32, // - hotel category
    pax: u16,            // - number of travelers; sum of adults and children
    adults: u16,         // - no.of adults
    children: u16,       // - no.of children
    room_name: String,   // - room name
    #[serde(with = "checkin_checkout_format")]
    checkin: Date, // - arrival date
    #[serde(with = "checkin_checkout_format")]
    checkout: Date, // - checkout date; please assume that checkout = checkin +1 day
    #[serde(rename = "price", with = "rust_decimal::serde::str")]
    price_per_person: Decimal, // - price for the stay PER PERSON
}

impl TryFrom<(input::Row, &hotels::HotelWithRooms)> for Output {
    type Error = ();

    fn try_from(value: (input::Row, &hotels::HotelWithRooms)) -> Result<Self, Self::Error> {
        let (input, hotel_with_rooms) = value;
        let hotel = hotel_with_rooms.hotel();
        let room = hotel_with_rooms
            .get_room(&input.room_code, &input.source)
            .unwrap();

        let pax = input.adults + input.children;
        let mut price_per_person = input.price / Decimal::from(pax);
        price_per_person = price_per_person.round_dp(2);
        price_per_person.rescale(2);

        Ok(Self {
            room_type_and_meal: RoomTypeWithMeal {
                room_type: input.room_type,
                meal: input.meal,
            },
            room_code: input.room_code,
            source: input.source,
            hotel_name: hotel.name.clone(),
            city_name: hotel.city_name.clone(),
            city_code: hotel.city_code.clone(),
            hotel_category: hotel.category,
            pax,
            adults: input.adults,
            children: input.children,
            room_name: room.name.clone(),
            checkin: input.checkin,
            checkout: input.checkin.next_day().unwrap(),
            price_per_person,
        })
    }
}

pub async fn sink_into_new_file<E>(
    output_stream: impl Stream<Item = Result<Output, E>> + Unpin,
    path: &Path,
) -> anyhow::Result<()>
where
    anyhow::Error: std::convert::From<E>,
{
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .context(format!("failed to create file: {}", path.display()))?;
    let wr = BufWriter::with_capacity(1024usize.pow(2), file);

    sink_into_writer(output_stream, wr).await
}

pub async fn sink_into_writer<E, W>(
    output_stream: impl Stream<Item = Result<Output, E>> + Unpin,
    wr: W,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
    anyhow::Error: std::convert::From<E>,
{
    let mut output_stream = output_stream;
    let mut serializer = AsyncWriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .create_serializer(wr);

    while let Some(res_row) = output_stream.next().await {
        let row = res_row?;
        serializer.serialize(row).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn serialize_output() {
        let expected_output = r#"room_type meal;room_code;source;hotel_name;city_name;city_code;hotel_category;pax;adults;children;room_name;checkin;checkout;price
EZ F;BER898;IHG;Crowne Plaza Berlin City Centre;Berlin;BER;4.0;1;1;0;Einzelzimmer;2018-07-21;2018-07-22;85.50
"#;

        let output = Output {
            room_type_and_meal: RoomTypeWithMeal {
                room_type: "EZ".to_string(),
                meal: "F".to_string(),
            },
            room_code: "BER898".to_string(),
            source: "IHG".to_string(),
            hotel_name: "Crowne Plaza Berlin City Centre".to_string(),
            city_name: "Berlin".to_string(),
            city_code: CityCode::from_str("BER").unwrap(),
            hotel_category: 4.0,
            pax: 1,
            adults: 1,
            children: 0,
            room_name: "Einzelzimmer".to_string(),
            checkin: Date::from_calendar_date(2018, time::Month::July, 21).unwrap(),
            checkout: Date::from_calendar_date(2018, time::Month::July, 22).unwrap(),
            price_per_person: Decimal::new(8550, 2),
        };

        let mut output_str = Vec::new();
        let wr = BufWriter::new(&mut output_str);

        let v: Vec<Result<_, csv_async::Error>> = vec![Ok(output)];

        sink_into_writer(tokio_stream::iter(v), wr).await.unwrap();

        assert_eq!(expected_output, String::from_utf8(output_str).unwrap());
    }
}
