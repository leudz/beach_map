use super::{Id, UntypedId};
use core::marker::PhantomData;
use core::num::NonZeroU32;
use serde::{ser::SerializeTupleStruct, Deserialize, Deserializer, Serialize, Serializer};

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut struc = serializer.serialize_tuple_struct("Id", 1)?;
        struc.serialize_field(&self.data)?;
        struc.end()
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (data,): (NonZeroU32,) = Deserialize::deserialize(deserializer)?;

        Ok(Id {
            data,
            phantom: PhantomData,
        })
    }
}

impl Serialize for UntypedId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UntypedId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id: Id<()> = Deserialize::deserialize(deserializer)?;

        Ok(UntypedId(id))
    }
}

#[test]
fn entity_id_serde() {
    use crate::BeachMap;

    let mut beach = BeachMap::<u32>::default();

    let id1 = beach.insert(5);
    check_roundtrip(id1, "[1]");
    let id2 = beach.insert(10);
    check_roundtrip(id2, "[2]");

    beach.remove(id1);

    let id3 = beach.insert(5);
    check_roundtrip(id3, "[16777217]");
    let id4 = beach.insert(10);
    check_roundtrip(id4, "[3]");
}

#[cfg(test)]
fn check_roundtrip(id: Id<u32>, expected: &str) {
    assert_eq!(expected, serde_json::to_string(&id).unwrap());
    let new_id: Id<u32> = serde_json::from_str(expected).unwrap();
    assert_eq!(id, new_id);
}
