use super::ID;
use serde::{ser::SerializeTupleStruct, Deserialize, Deserializer, Serialize, Serializer};

impl<V: Serialize> Serialize for ID<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple_struct("ID", 2)?;
        tup.serialize_field(&self.index)?;
        tup.serialize_field(&(self.version))?;
        tup.end()
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for ID<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (index, version): (usize, V) = Deserialize::deserialize(deserializer)?;
        Ok(ID { index, version })
    }
}

#[test]
fn entity_id_serde() {
    use crate::BeachMap;

    let mut beach = BeachMap::<u32, _>::default();

    let id1 = beach.insert(5);
    check_roundtrip(id1, "[0,0]");
    let id2 = beach.insert(10);
    check_roundtrip(id2, "[1,0]");

    beach.remove(id1);

    let id3 = beach.insert(5);
    check_roundtrip(id3, "[0,1]");
    let id4 = beach.insert(10);
    check_roundtrip(id4, "[2,0]");
}

#[cfg(test)]
fn check_roundtrip(id: ID<u32>, expected: &str) {
    assert_eq!(expected, serde_json::to_string(&id).unwrap());
    let new_id: ID<u32> = serde_json::from_str(expected).unwrap();
    assert_eq!(id, new_id);
}
