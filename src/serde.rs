pub mod checkin_checkout_format {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::Date;

    const IN_FORMAT: &[time::format_description::FormatItem] =
        time::macros::format_description!("[year][month][day]");
    const OUT_FORMAT: &[time::format_description::FormatItem] =
        time::macros::format_description!("[year]-[month]-[day]");

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .format(&OUT_FORMAT)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse(&s, &IN_FORMAT).map_err(serde::de::Error::custom)
    }
}
