use c3p0_common::json::codec::JsonCodec;

pub trait C3p0JsonAsync<DATA, CODEC>: Clone
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
{
    type CONN;

    fn codec(&self) -> &CODEC;

    // ToDo: this does not work due to a bug in rustc.
    // See: https://github.com/rust-lang/rust/issues/63033
    /*
    async fn create_table_if_not_exists(&self, conn: &Self::CONN) -> Result<(), C3p0Error>;

    async fn drop_table_if_exists(&self, conn: &Self::CONN, cascade: bool)
        -> Result<(), C3p0Error>;

    async fn count_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    async fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    async fn fetch_all(&self, conn: &Self::CONN) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_all_for_update(
        &self,
        conn: &Self::CONN,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error>;

    async fn delete(&self, conn: &Self::CONN, obj: &Model<DATA>) -> Result<u64, C3p0Error>;

    async fn delete_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    async fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    async fn save(&self, conn: &Self::CONN, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    async fn update(&self, conn: &Self::CONN, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;

    */
}
