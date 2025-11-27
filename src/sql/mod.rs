use sqlx::{Database, Decode, Encode, Type};

pub mod player_join;

pub struct SqlU64(u64);

impl SqlU64 {
    pub fn new(num: u64) -> Self {
        Self(num)
    }

    pub fn from_db(num: i64) -> Self {
        Self(num as u64)
    }
    
    pub fn to_db(&self) -> i64 {
        self.0 as i64
    }
}

impl From<u64> for SqlU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl<DB> Type<DB> for SqlU64 
where
    DB: Database,
    i64: Type<DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        <i64 as Type<DB>>::type_info()
    }
}

impl<'r, DB> Decode<'r, DB> for SqlU64 
where
    DB: Database,
    i64: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = i64::decode(value)?;
        Ok(Self(value as u64))
    }
}

impl<'r, DB> Encode<'r, DB> for SqlU64 
where
    DB: Database,
    i64: Encode<'r, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'r>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        (self.0 as i64).encode(buf)
    }
}
