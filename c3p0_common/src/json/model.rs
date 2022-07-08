use serde::{Deserialize, Serialize};

pub type IdType = i64;
pub type VersionType = i32;
pub type EpochMillisType = i64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Model<Data>
where
    Data: Clone + serde::ser::Serialize + Send,
{
    pub id: IdType,
    pub version: VersionType,
    pub create_epoch_millis: EpochMillisType,
    pub update_epoch_millis: EpochMillisType,
    #[serde(bound(deserialize = "Data: serde::Deserialize<'de>"))]
    pub data: Data,
}

impl<Data> Model<Data>
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    pub fn into_new(self) -> NewModel<Data> {
        NewModel::new(self.data)
    }

    pub fn from_new(id: IdType, create_epoch_millis: EpochMillisType, model: NewModel<Data>) -> Model<Data> {
        Model { id, version: model.version, create_epoch_millis, update_epoch_millis: create_epoch_millis, data: model.data }
    }

    pub fn into_new_version(self, update_epoch_millis: EpochMillisType) -> Model<Data> {
        Model { id: self.id, version: self.version + 1, create_epoch_millis: self.create_epoch_millis, update_epoch_millis, data: self.data }
    }
}

impl<'a, Data> From<&'a Model<Data>> for &'a IdType
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn from(model: &'a Model<Data>) -> Self {
        &model.id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewModel<Data>
where
    Data: Clone + serde::ser::Serialize,
{
    pub version: VersionType,
    #[serde(bound(deserialize = "Data: serde::Deserialize<'de>"))]
    pub data: Data,
}

impl<Data> NewModel<Data>
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    pub fn new(data: Data) -> Self {
        NewModel { version: 0, data }
    }
}

impl<Data> Default for NewModel<Data>
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Default,
{
    fn default() -> Self {
        NewModel::new(Data::default())
    }
}

impl<Data> From<Data> for NewModel<Data>
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    fn from(data: Data) -> Self {
        NewModel::new(data)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[test]
    fn model_should_be_serializable() -> Result<(), Box<dyn std::error::Error>> {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
            create_epoch_millis: 0,
            update_epoch_millis: 0,
        };

        let serialize = serde_json::to_string(&model)?;
        let deserialize: Model<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.id, deserialize.id);
        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);

        Ok(())
    }

    #[test]
    fn new_model_should_be_serializable() -> Result<(), Box<dyn std::error::Error>> {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        let serialize = serde_json::to_string(&model)?;
        let deserialize: NewModel<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);
        Ok(())
    }

    #[test]
    fn model_should_impl_debug_if_data_is_debug() {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
            create_epoch_millis: 0,
            update_epoch_millis: 0,
        };

        println!("Debug model: {:?}", model);
    }

    #[test]
    fn new_model_should_impl_debug_if_data_is_debug() {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        println!("Debug model: {:?}", model);
    }

    #[test]
    fn new_model_from_model_should_have_new_version() {
        
        let model = Model {
            id: 10,
            version: 10,
            data: SimpleData {
                name: "test".to_owned(),
            },
            create_epoch_millis: 0,
            update_epoch_millis: 0,
        };

        let new_model = model.clone().into_new();

        assert_eq!(model.data, new_model.data);
        assert_eq!(new_model.version, 0);
    }

    #[test]
    fn should_build_new_model_version() {
        
        let model = Model {
            id: 10,
            version: 10,
            data: SimpleData {
                name: "test".to_owned(),
            },
            create_epoch_millis: 0,
            update_epoch_millis: 0,
        };

        let new_update_epoch_millis = 111;

        let new_model_version = model.clone().into_new_version(new_update_epoch_millis);

        assert_eq!(model.data, new_model_version.data);
        assert_eq!(model.id, new_model_version.id);
        assert_eq!(model.create_epoch_millis, new_model_version.create_epoch_millis);
        assert_eq!(new_update_epoch_millis, new_model_version.update_epoch_millis);
        assert_eq!(model.version + 1, new_model_version.version);
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
