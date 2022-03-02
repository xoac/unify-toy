use csv_async::{AsyncReaderBuilder, DeserializeRecordsIntoStream};
use std::path::Path;
use time::Date;
use tokio::{fs::File, io::BufReader};

use crate::serde::checkin_checkout_format;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::city_code::CityCode;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Row {
    pub city_code: CityCode,
    pub hotel_code: String, //- hotel code
    pub room_type: String,  //- room type code
    pub room_code: String,  //- room code
    pub meal: String,       //- meal code
    #[serde(with = "checkin_checkout_format")]
    pub checkin: Date, //- arrival date
    pub adults: u16,        //- no. of adults
    pub children: u16,      //- no. of children
    pub price: Decimal,     //- price for the stay
    pub source: String,     //- name of the price source
}

pub type InputStream<'r, R> = DeserializeRecordsIntoStream<'r, R, Row>;

/// AsyncReader from file with buffered data
pub async fn stream_from_file(
    path: &Path,
) -> std::io::Result<InputStream<'static, BufReader<File>>> {
    let file = File::open(path).await?;
    let buf_rdr = BufReader::with_capacity(1_048_576, file);
    Ok(stream_from_reader(buf_rdr))
}

fn stream_from_reader<'r, R>(rdr: R) -> DeserializeRecordsIntoStream<'r, R, Row>
where
    R: tokio::io::AsyncRead + Send + Unpin + 'r,
{
    let mut b = AsyncReaderBuilder::new();
    b.delimiter(b'|');
    b.has_headers(true);
    // TODO:
    // 1. make sure all options are set correctly

    b.create_deserializer(rdr).into_deserialize()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use tokio_stream::StreamExt;
    #[tokio::test]
    async fn deserialize_input() {
        let input =
            r#"city_code|hotel_code|room_type|room_code|meal|checkin|adults|children|price|source
BER|BER00002|EZ|BER898|F|20180721|1|0|85.50|IHG
BER|BER00002|EZ|BER898|F|20180722|1|0|78.00|IHG
BER|BER00002|EZ|BER898|F|20180723|1|0|85.50|IHG
BER|BER00003|DZ|BER848|U|20180721|2|0|101.59|MARR
BER|BER00003|DZ|BER848|U|20180722|2|0|109.46|MARR
BER|BER00003|DZ|BER848|U|20180723|2|1|176.01|MARR
"#
            .as_bytes();

        let deser = stream_from_reader(input);

        let rows = deser
            .map(|x| {
                println!("{:?}", x);
                x
            })
            .collect::<Result<Vec<Row>, _>>()
            .await
            .unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(
            rows[0],
            Row {
                city_code: CityCode::from_str("BER").unwrap(),
                hotel_code: "BER00002".into(),
                room_type: "EZ".into(),
                room_code: "BER898".into(),
                meal: "F".into(),
                checkin: Date::from_calendar_date(2018, time::Month::July, 21).unwrap(),
                adults: 1,
                children: 0,
                price: Decimal::new(8550, 2),
                source: "IHG".into()
            }
        )
    }
}
