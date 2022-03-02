//! # Summary
//! Task CLI toy application to add details to csv files
//! # Examples
//! Code can be run with
//! ```sh
//! cargo run -- --hotels examples/hotels.json --room-names examples/room_names.csv --input examples/input.csv --output expected.csv
//! ````

use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
use tokio_stream::StreamExt;
use unify_toy::hotels::HotelWithRooms;
use unify_toy::Args;
use unify_toy::{hotels, input, output, room_names};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let hotel_stream = hotels::stream_from_file(&args.hotels).await?;

    let room_names = room_names::stream_from_file(&args.room_names).await?;

    /* Read necessery file to add details to output */
    let mut hotels = HashMap::with_capacity(1024 * 1024);
    let mut hotel_stream = Box::pin(hotel_stream);
    while let Some(res_item) = hotel_stream.next().await {
        let hotel = res_item?;

        let old = hotels.insert(hotel.id.clone(), HotelWithRooms::new(hotel));
        assert!(old.is_none());
    }

    let mut room_names = Box::pin(room_names);
    while let Some(res_item) = room_names.next().await {
        let room_detail = res_item?;

        let hotel = hotels.get_mut(&room_detail.hotel_code).ok_or_else(|| {
            anyhow::Error::msg("inconsist input files. Expected room_names be subset of hotels")
        })?;

        hotel.insert_room(room_detail);
    }

    /* Create read input and map it to output */
    let in_stream = input::stream_from_file(&args.input)
        .await
        .context("failed to read input file from csv file")?;

    let output_stream: async_stream::AsyncStream<anyhow::Result<output::Output>, _> = async_stream::try_stream! {
        let mut in_stream = Box::pin(in_stream);
        while let Some(res_input_row) = in_stream.next().await {
            let in_row = res_input_row.map_err(anyhow::Error::from)?;

            // TODO: input looks like sorted by hotel_code, if this is true `hotel` here could
            //       be cached
            let hotel = hotels.get(&in_row.hotel_code).ok_or_else(|| {
                anyhow::Error::msg("inconsist input files. Expected input to be subset of hotels")
            })?;

            let output = output::Output::try_from((in_row, hotel)).unwrap();
            yield output;
        }
    };

    output::sink_into_new_file(Box::pin(output_stream), &args.output).await?;

    Ok(())
}
