use crate::{city_code::CityCode, room_names::RoomName};
use async_stream::try_stream;
use serde::{ser::Error as SerError, Deserialize};
use serde_json::{Error as JsonError, Result as JsonResult};
use std::{collections::HashMap, io, path::Path};
use tokio::{
    fs::File,
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
};
use tokio_stream::Stream;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Hotel {
    pub id: String,
    pub city_code: CityCode,
    pub name: String,
    pub category: f32,
    pub country_code: String, // FIXME validate for corectness
    #[serde(rename = "city")]
    pub city_name: String,
}

pub async fn stream_from_file(path: &Path) -> io::Result<impl Stream<Item = JsonResult<Hotel>>> {
    let file = File::open(path).await?;
    let rdr = BufReader::with_capacity(1024usize.pow(3), file);

    Ok(stream_from_reader(rdr))
}

fn stream_from_reader<R>(rdr: R) -> impl Stream<Item = JsonResult<Hotel>>
where
    R: AsyncBufRead + Unpin + Send,
{
    try_stream!(
        let mut lines = rdr.lines();
        while let Some(line) = lines.next_line().await.map_err(JsonError::custom)? {
            if line.is_empty() {
                continue;
            }
            let deserialize = serde_json::from_str(&line)?;
            yield deserialize;
        }
    )
}

#[derive(Debug, Clone)]
pub struct HotelWithRooms {
    hotel: Hotel,
    // TODO: (String, Sting) should be named types like (RoomCode, RoomSource)
    // TODO: possible optimisation with keyPair: https://stackoverflow.com/a/45795699/5190508
    // FIXME: possible incorrect implementation
    rooms: HashMap<(String, String), RoomName>,
}

impl HotelWithRooms {
    pub fn new(hotel: Hotel) -> Self {
        Self {
            hotel,
            rooms: Default::default(),
        }
    }

    pub fn hotel(&self) -> &Hotel {
        &self.hotel
    }

    #[inline] // inline will allow compiler to optimise "make borrow happy `clone()`s"
    pub fn get_room(&self, room_code: &str, room_source: &str) -> Option<&RoomName> {
        self.rooms
            .get(&(room_code.to_owned(), room_source.to_owned()))
    }

    pub fn insert_room(&mut self, room: RoomName) {
        assert_eq!(self.hotel.id, room.hotel_code);
        // we do not care about doubles data
        let old = self
            .rooms
            .insert((room.code.clone(), room.source.clone()), room);

        // FIXME: user input should be handled as Error not panic
        assert!(old.is_none());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use tokio_stream::StreamExt;
    #[tokio::test]
    async fn deserialize_input() {
        let input = r#"
        {"id": "BER00002", "city_code": "BER", "name": "Crowne Plaza Berlin City Centre", "category": 4.0, "country_code": "DE", "city": "Berlin" }
        {"id": "BER00003", "city_code": "BER", "name": "Berlin Marriott Hotel", "category": 5.0, "country_code": "DE", "city": "Berlin" }
"#.as_bytes();

        let stream = stream_from_reader(input);

        let rows = stream
            .map(|x| {
                println!("{:?}", x);
                x
            })
            .collect::<Result<Vec<Hotel>, _>>()
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            Hotel {
                id: "BER00002".to_owned(),
                city_code: CityCode::from_str("BER").unwrap(),
                name: "Crowne Plaza Berlin City Centre".to_owned(),
                category: 4.0,
                country_code: "DE".to_owned(),
                city_name: "Berlin".to_owned(),
            }
        )
    }
}
