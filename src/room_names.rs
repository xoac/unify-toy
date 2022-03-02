use csv_async::Result as CsvResult;
use serde::Deserialize;
use std::{io, path::Path};
use tokio::{
    fs::File,
    io::{AsyncRead, BufReader},
};
use tokio_stream::Stream;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct RoomName {
    pub hotel_code: String, // - hotel code
    pub source: String,     // - name of the price source
    #[serde(rename = "room_name")]
    pub name: String, // - room name
    #[serde(rename = "room_code")]
    pub code: String, // - room code
}

pub async fn stream_from_file(path: &Path) -> io::Result<impl Stream<Item = CsvResult<RoomName>>> {
    let file = File::open(path).await?;
    let rdr = BufReader::with_capacity(1024usize.pow(3), file);

    Ok(stream_from_reader(rdr))
}

fn stream_from_reader<'r, R>(rdr: R) -> impl Stream<Item = CsvResult<RoomName>> + 'r
where
    R: AsyncRead + Unpin + Send + 'r,
{
    let async_deserialize = csv_async::AsyncReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .create_deserializer(rdr);

    async_deserialize.into_deserialize()
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_stream::StreamExt;
    #[tokio::test]
    async fn deserialize_input() {
        let input = r#"
BER00003|MARR|Single Standard|BER849
BER00003|MARR|Deluxe King|BER848
BER00003|DOTW|SINGLE DELUXE|BER848
BER00002|GTA|Standard|BER898
BER00002|IHG|Einzelzimmer|BER898
BER00002|MARR|Deluxe King Extra|BER848
"#
        .as_bytes();

        let deser = stream_from_reader(input);

        let rows = deser
            .map(|x| {
                println!("{:?}", x);
                x
            })
            .collect::<Result<Vec<RoomName>, _>>()
            .await
            .unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(
            rows[0],
            RoomName {
                hotel_code: "BER00003".to_owned(),
                source: "MARR".to_owned(),
                name: "Single Standard".to_owned(),
                code: "BER849".to_owned()
            }
        )
    }
}
