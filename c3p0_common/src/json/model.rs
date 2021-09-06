use serde::{Deserialize, Serialize};

pub type IdType = i64;
pub type VersionType = i32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Model<Data>
where
    Data: Clone + serde::ser::Serialize + Send,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "Data: serde::Deserialize<'de>"))]
    pub data: Data,
}

impl<Data> Model<Data>
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send,
{
    pub fn into_new(self) -> NewModel<Data> {
        NewModel {
            version: 0,
            data: self.data,
        }
    }
}

impl<'a, Data> From<&'a Model<Data>> for &'a IdType
where
    Data: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send, {
        
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

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
