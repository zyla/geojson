// Copyright 2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::json::{self, Deserialize, Deserializer, JsonObject, JsonValue, Serialize, Serializer};
use crate::serde;
use crate::{Error, Feature, FeatureCollection, Geometry, Position};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

/// GeoJSON Objects
///
/// ```
/// use std::convert::TryInto;
/// use geojson::{Feature, GeoJson, Geometry, Value};
/// use serde_json::json;
/// let json_value = json!({
///     "type": "Feature",
///     "geometry": {
///         "type": "Point",
///         "coordinates": [102.0, 0.5]
///     },
///     "properties": null,
/// });
/// let feature: Feature = json_value.try_into().unwrap();
///
/// // Easily convert a feature to a GeoJson
/// let geojson: GeoJson = feature.into();
/// // and back again
/// let feature2: Feature = geojson.try_into().unwrap();
/// ```
/// [GeoJSON Format Specification § 3](https://tools.ietf.org/html/rfc7946#section-3)
#[derive(Clone, Debug, PartialEq)]
pub enum GeoJson<Pos> {
    Geometry(Geometry<Pos>),
    Feature(Feature<Pos>),
    FeatureCollection(FeatureCollection<Pos>),
}

impl<'a, P: Position> From<&'a GeoJson<P>> for JsonObject {
    fn from(geojson: &'a GeoJson<P>) -> JsonObject {
        match *geojson {
            GeoJson::Geometry(ref geometry) => geometry.into(),
            GeoJson::Feature(ref feature) => feature.into(),
            GeoJson::FeatureCollection(ref fc) => fc.into(),
        }
    }
}

impl<P: Position> From<GeoJson<P>> for JsonValue {
    fn from(geojson: GeoJson<P>) -> JsonValue {
        match geojson {
            GeoJson::Geometry(geometry) => JsonValue::Object(JsonObject::from(&geometry)),
            GeoJson::Feature(feature) => JsonValue::Object(JsonObject::from(&feature)),
            GeoJson::FeatureCollection(fc) => JsonValue::Object(JsonObject::from(&fc)),
        }
    }
}

impl<P: Position> From<Geometry<P>> for GeoJson<P> {
    fn from(geometry: Geometry<P>) -> Self {
        GeoJson::Geometry(geometry)
    }
}

impl<P: Position> From<Feature<P>> for GeoJson<P> {
    fn from(feature: Feature<P>) -> Self {
        GeoJson::Feature(feature)
    }
}

impl<P: Position> From<FeatureCollection<P>> for GeoJson<P> {
    fn from(feature_collection: FeatureCollection<P>) -> GeoJson<P> {
        GeoJson::FeatureCollection(feature_collection)
    }
}

impl<P: Position> TryFrom<GeoJson<P>> for Geometry<P> {
    type Error = Error<P>;
    fn try_from(value: GeoJson<P>) -> Result<Self, Self::Error> {
        match value {
            GeoJson::Geometry(g) => Ok(g),
            GeoJson::Feature(_) => Err(Error::ExpectedType {
                expected: "Geometry".to_string(),
                actual: "Feature".to_string(),
            }),
            GeoJson::FeatureCollection(_) => Err(Error::ExpectedType {
                expected: "Geometry".to_string(),
                actual: "FeatureCollection".to_string(),
            }),
        }
    }
}

impl<P: Position> TryFrom<GeoJson<P>> for Feature<P> {
    type Error = Error<P>;
    fn try_from(value: GeoJson<P>) -> Result<Self, Self::Error> {
        match value {
            GeoJson::Geometry(_) => Err(Error::ExpectedType {
                expected: "Feature".to_string(),
                actual: "Geometry".to_string(),
            }),
            GeoJson::Feature(f) => Ok(f),
            GeoJson::FeatureCollection(_) => Err(Error::ExpectedType {
                expected: "Feature".to_string(),
                actual: "FeatureCollection".to_string(),
            }),
        }
    }
}

impl<P: Position> TryFrom<GeoJson<P>> for FeatureCollection<P> {
    type Error = Error<P>;
    fn try_from(value: GeoJson<P>) -> Result<Self, Self::Error> {
        match value {
            GeoJson::Geometry(_) => Err(Error::ExpectedType {
                expected: "FeatureCollection".to_string(),
                actual: "Geometry".to_string(),
            }),
            GeoJson::Feature(_) => Err(Error::ExpectedType {
                expected: "FeatureCollection".to_string(),
                actual: "Feature".to_string(),
            }),
            GeoJson::FeatureCollection(f) => Ok(f),
        }
    }
}

impl<P: Position> GeoJson<P> {
    pub fn from_json_object(object: JsonObject) -> Result<Self, Error<P>> {
        Self::try_from(object)
    }
}

impl<P: Position> GeoJson<P> {
    /// Converts a JSON Value into a GeoJson object.
    ///
    /// # Example
    /// ```
    /// use std::convert::TryInto;
    /// use geojson::{Feature, GeoJson, Geometry, Value};
    /// use serde_json::json;
    ///
    /// let json_value = json!({
    ///     "type": "Feature",
    ///     "geometry": {
    ///         "type": "Point",
    ///         "coordinates": [102.0, 0.5]
    ///     },
    ///     "properties": null,
    /// });
    ///
    /// assert!(json_value.is_object());
    ///
    /// let geojson: GeoJson<_> = json_value.try_into().unwrap();
    ///
    /// assert_eq!(
    ///     geojson,
    ///     GeoJson::Feature(Feature {
    ///         bbox: None,
    ///         geometry: Some(Geometry::new(Value::Point((102.0, 0.5)))),
    ///         id: None,
    ///         properties: None,
    ///         foreign_members: None,
    ///     })
    /// );
    /// ```
    pub fn from_json_value(value: JsonValue) -> Result<Self, Error<P>> {
        Self::try_from(value)
    }

    /// Convience method to convert to a JSON Value. Uses `From`.
    /// ```
    /// use std::convert::TryFrom;
    /// use geojson::GeoJson;
    /// use serde_json::json;
    ///
    /// let geojson = GeoJson::try_from( json!({
    ///        "type": "Feature",
    ///        "geometry": {
    ///            "type": "Point",
    ///            "coordinates": [102.0, 0.5]
    ///        },
    ///        "properties": {},
    ///     })).unwrap();
    ///
    /// let json_value = geojson.to_json_value();
    /// assert_eq!(json_value,
    ///     json!({
    ///        "type": "Feature",
    ///        "geometry": {
    ///            "type": "Point",
    ///            "coordinates": [102.0, 0.5]
    ///        },
    ///        "properties": {},
    ///     })
    ///    );
    /// ```
    pub fn to_json_value(self) -> JsonValue {
        JsonValue::from(self)
    }

    // Deserialize a GeoJson object from an IO stream of JSON
    pub fn from_reader<R>(rdr: R) -> Result<Self, serde_json::Error>
    where
        R: std::io::Read,
    {
        serde_json::from_reader(rdr)
    }
}

impl<P: Position> TryFrom<JsonObject> for GeoJson<P> {
    type Error = Error<P>;

    fn try_from(object: JsonObject) -> Result<Self, Self::Error> {
        let type_ = match object.get("type") {
            Some(json::JsonValue::String(t)) => Type::from_str(t),
            _ => return Err(Error::GeometryUnknownType("type".to_owned())),
        };
        let type_ = type_.ok_or(Error::EmptyType)?;
        match type_ {
            Type::Feature => Feature::try_from(object).map(GeoJson::Feature),
            Type::FeatureCollection => {
                FeatureCollection::try_from(object).map(GeoJson::FeatureCollection)
            }
            _ => Geometry::<P>::try_from(object).map(GeoJson::Geometry),
        }
    }
}

impl<P: Position> TryFrom<JsonValue> for GeoJson<P> {
    type Error = Error<P>;

    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        if let JsonValue::Object(obj) = value {
            Self::try_from(obj)
        } else {
            Err(Error::GeoJsonExpectedObject(value))
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Type {
    Point,
    MultiPoint,
    LineString,
    MultiLineString,
    Polygon,
    MultiPolygon,
    GeometryCollection,
    Feature,
    FeatureCollection,
}

impl Type {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Point" => Some(Type::Point),
            "MultiPoint" => Some(Type::MultiPoint),
            "LineString" => Some(Type::LineString),
            "MultiLineString" => Some(Type::MultiLineString),
            "Polygon" => Some(Type::Polygon),
            "MultiPolygon" => Some(Type::MultiPolygon),
            "GeometryCollection" => Some(Type::GeometryCollection),
            "Feature" => Some(Type::Feature),
            "FeatureCollection" => Some(Type::FeatureCollection),
            _ => None,
        }
    }
}

impl<P: Position> Serialize for GeoJson<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        JsonObject::from(self).serialize(serializer)
    }
}

impl<'de, Pos: Position> Deserialize<'de> for GeoJson<Pos> {
    fn deserialize<D>(deserializer: D) -> Result<GeoJson<Pos>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;

        let val = JsonObject::deserialize(deserializer)?;

        GeoJson::from_json_object(val).map_err(|e| D::Error::custom(e.to_string()))
    }
}

/// # Example
///```
/// use geojson::GeoJson;
/// use std::str::FromStr;
///
/// let geojson_str = r#"{
///   "type": "FeatureCollection",
///   "features": [
///     {
///       "type": "Feature",
///       "properties": {},
///       "geometry": {
///         "type": "Point",
///         "coordinates": [
///           -0.13583511114120483,
///           51.5218870403801
///         ]
///       }
///     }
///   ]
/// }
/// "#;
/// let geo_json = GeoJson::from_str(&geojson_str).unwrap();
/// if let GeoJson::FeatureCollection(collection) = geo_json {
///     assert_eq!(1, collection.features.len());
/// } else {
///     panic!("expected feature collection");
/// }
/// ```
impl<P: Position> FromStr for GeoJson<P> {
    type Err = Error<P>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let object = get_object(s)?;

        GeoJson::from_json_object(object)
    }
}

fn get_object<P: Position>(s: &str) -> Result<json::JsonObject, Error<P>> {
    match ::serde_json::from_str(s) {
        Ok(json::JsonValue::Object(object)) => Ok(object),
        Ok(other) => Err(Error::ExpectedObjectValue(other)),
        Err(serde_error) => Err(Error::MalformedJson(serde_error)),
    }
}

impl<P: Position> fmt::Display for GeoJson<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ::serde_json::to_string(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

impl<P: Position> fmt::Display for Feature<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ::serde_json::to_string(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

impl<P: Position> fmt::Display for Geometry<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ::serde_json::to_string(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

impl<P: Position> fmt::Display for FeatureCollection<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ::serde_json::to_string(self)
            .map_err(|_| fmt::Error)
            .and_then(|s| f.write_str(&s))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Feature, GeoJson, Geometry, Value};
    use serde_json::json;
    use std::convert::TryInto;
    use std::str::FromStr;

    #[test]
    fn test_geojson_from_reader() {
        let json_str = r#"{
            "type": "Feature",
            "geometry": {
                    "type": "Point",
                    "coordinates": [102.0, 0.5]
            },
            "properties": null
        }"#;

        let g1 = GeoJson::from_reader(json_str.as_bytes()).unwrap();

        let json_value = json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [102.0, 0.5]
            },
            "properties": null,
        });

        let g2: GeoJson<(f64, f64)> = json_value.try_into().unwrap();

        assert_eq!(g1, g2);
    }

    #[test]
    fn test_geojson_from_value() {
        let json_value = json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [102.0, 0.5]
            },
            "properties": null,
        });

        assert!(json_value.is_object());

        let geojson: GeoJson<(f64, f64)> = json_value.try_into().unwrap();

        assert_eq!(
            geojson,
            GeoJson::Feature(Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::Point((102.0, 0.5)))),
                id: None,
                properties: None,
                foreign_members: None,
            })
        );
    }

    #[test]
    fn test_invalid_json() {
        let geojson_str = r#"{
           "type": "FeatureCollection",
           "features": [
             !INTENTIONAL_TYPO! {
               "type": "Feature",
               "properties": {},
               "geometry": {
                 "type": "Point",
                 "coordinates": [
                   -0.13583511114120483,
                   51.5218870403801
                 ]
               }
             }
           ]
        }"#;
        assert!(matches!(
            GeoJson::<(f64, f64)>::from_str(geojson_str),
            Err(Error::MalformedJson(_))
        ))
    }
}
