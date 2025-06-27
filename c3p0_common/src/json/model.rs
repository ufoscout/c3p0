use serde::{Deserialize, Serialize};

use crate::{DataType, IdType};

use super::types::{EpochMillisType, VersionType};

/// A model for a database table.
/// This is used to retrieve and update an entry in a database table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Model<Id, Data> {
    /// The unique identifier of the model.
    pub id: Id,
    /// The version of the model used for optimistic locking.
    pub version: VersionType,
    /// The epoch millis when the model was created.
    #[serde(default)]
    pub create_epoch_millis: EpochMillisType,
    /// The epoch millis when the model was last updated.
    #[serde(default)]
    pub update_epoch_millis: EpochMillisType,
    /// The data of the model.
    pub data: Data,
}

impl<Id: IdType, Data: DataType> Model<Id, Data> {
    /// Converts the current `Model` instance into a `NewModel` instance,
    /// resetting the version to the initial state while retaining the data.
    pub fn into_new(self) -> NewModel<Data> {
        NewModel::new(self.data)
    }

    /// Creates a new `Model` instance from a `NewModel` instance.
    ///
    /// - `id`: The unique identifier of the model.
    /// - `create_epoch_millis`: The epoch millis when the model was created.
    /// - `model`: The `NewModel` instance to create the `Model` instance from.
    ///
    /// Returns a `Model` instance with the version set to the initial state,
    /// the create and update epoch millis set to the given `create_epoch_millis`,
    /// and the data set to the data of the `model` parameter.
    pub fn from_new(
        id: Id,
        create_epoch_millis: EpochMillisType,
        model: NewModel<Data>,
    ) -> Model<Id, Data> {
        Model {
            id,
            version: model.version,
            create_epoch_millis,
            update_epoch_millis: create_epoch_millis,
            data: model.data,
        }
    }

    /// Creates a new `Model` instance from the current `Model` instance,
    /// incrementing the version by one and updating the update epoch millis
    /// to the given `update_epoch_millis`.
    ///
    /// - `update_epoch_millis`: The epoch millis when the model was last updated.
    ///
    /// Returns a `Model` instance with the version incremented by one,
    /// the create epoch millis unchanged, the update epoch millis set to
    /// the given `update_epoch_millis`, and the data unchanged.
    pub fn into_new_version(self, update_epoch_millis: EpochMillisType) -> Model<Id, Data> {
        Model {
            id: self.id,
            version: self.version + 1,
            create_epoch_millis: self.create_epoch_millis,
            update_epoch_millis,
            data: self.data,
        }
    }
}

/// A new model for a database table.
/// This is used to create a new entry in a database table.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewModel<Data> {
    pub version: VersionType,
    pub data: Data,
}

impl<Data: DataType> NewModel<Data> {
    /// Creates a new `NewModel` instance from a `Data` value.
    /// Sets the version to 0.
    pub fn new(data: Data) -> Self {
        NewModel { version: 0, data }
    }
}

impl<Data: DataType + Default> Default for NewModel<Data> {
    fn default() -> Self {
        NewModel::new(Data::default())
    }
}

impl<Data> From<Data> for NewModel<Data>
where
    Data: DataType,
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
        let deserialize: Model<i64, SimpleData> = serde_json::from_str(&serialize)?;

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

        println!("Debug model: {model:?}");
    }

    #[test]
    fn new_model_should_impl_debug_if_data_is_debug() {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        println!("Debug model: {model:?}");
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
        assert_eq!(
            model.create_epoch_millis,
            new_model_version.create_epoch_millis
        );
        assert_eq!(
            new_update_epoch_millis,
            new_model_version.update_epoch_millis
        );
        assert_eq!(model.version + 1, new_model_version.version);
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
