// Copyright 2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeMap;

#[cfg(not(feature = "with-serde"))]
use ::json::ToJson;
#[cfg(feature = "with-serde")]
use ::json::{Serialize, Deserialize, Serializer, Deserializer, SerdeError};

use ::json::{JsonValue, JsonObject, json_val};

use ::{Bbox, Crs, Error, LineStringType, PointType, PolygonType, FromObject, util};


/// The underlying Geometry value
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Point
    ///
    /// [GeoJSON Format Specification § 2.1.2]
    /// (http://geojson.org/geojson-spec.html#point)
    Point(PointType),

    /// MultiPoint
    ///
    /// [GeoJSON Format Specification § 2.1.3]
    /// (http://geojson.org/geojson-spec.html#multipoint)
    MultiPoint(Vec<PointType>),

    /// LineString
    ///
    /// [GeoJSON Format Specification § 2.1.4]
    /// (http://geojson.org/geojson-spec.html#linestring)
    LineString(LineStringType),

    /// MultiLineString
    ///
    /// [GeoJSON Format Specification § 2.1.5]
    /// (http://geojson.org/geojson-spec.html#multilinestring)
    MultiLineString(Vec<LineStringType>),

    /// Polygon
    ///
    /// [GeoJSON Format Specification § 2.1.6]
    /// (http://geojson.org/geojson-spec.html#polygon)
    Polygon(PolygonType),

    /// MultiPolygon
    ///
    /// [GeoJSON Format Specification § 2.1.7]
    /// (http://geojson.org/geojson-spec.html#multipolygon)
    MultiPolygon(Vec<PolygonType>),

    /// GeometryCollection
    ///
    /// [GeoJSON Format Specification § 2.1.8]
    /// (http://geojson.org/geojson-spec.html#geometry-collection)
    GeometryCollection(Vec<Geometry>),
}

#[cfg(not(feature = "with-serde"))]
impl ToJson for Value {
    fn to_json(&self) -> JsonValue {
        return match *self {
            Value::Point(ref x) => json_val(x),
            Value::MultiPoint(ref x) => json_val(x),
            Value::LineString(ref x) => json_val(x),
            Value::MultiLineString(ref x) => json_val(x),
            Value::Polygon(ref x) => json_val(x),
            Value::MultiPolygon(ref x) => json_val(x),
            Value::GeometryCollection(ref x) => json_val(x),
        };
    }
}

#[cfg(feature = "with-serde")]
impl<'a> From<&'a Value> for JsonValue {
    fn from(value: &'a Value) -> JsonValue {
        return match *value {
            Value::Point(ref x) => json_val(x),
            Value::MultiPoint(ref x) => json_val(x),
            Value::LineString(ref x) => json_val(x),
            Value::MultiLineString(ref x) => json_val(x),
            Value::Polygon(ref x) => json_val(x),
            Value::MultiPolygon(ref x) => json_val(x),
            Value::GeometryCollection(ref x) => json_val(x),
        };
    }
}

#[cfg(feature = "with-serde")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer {
        JsonValue::from(self).serialize(serializer)
    }
}

/// Geometry Objects
///
/// [GeoJSON Format Specification § 2.1]
/// (http://geojson.org/geojson-spec.html#geometry-objects)
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub bbox: Option<Bbox>,
    pub value: Value,
    pub crs: Option<Crs>,
}

impl Geometry {
    /// Returns a new `Geometry` with the specified `value`. `bbox` and `crs` will be set to
    /// `None`.
    pub fn new(value: Value) -> Self {
        Geometry {
            bbox: None,
            value: value,
            crs: None,
        }
    }
}

impl<'a> From<&'a Geometry> for JsonObject {
    fn from(geometry: &'a Geometry) -> JsonObject {
        let mut map = BTreeMap::new();
        if let Some(ref crs) = geometry.crs {
            map.insert(String::from("crs"), json_val(crs));
        }
        if let Some(ref bbox) = geometry.bbox {
            map.insert(String::from("bbox"), json_val(bbox));
        }

        let ty = String::from(match geometry.value {
            Value::Point(..) => "Point",
            Value::MultiPoint(..) => "MultiPoint",
            Value::LineString(..) => "LineString",
            Value::MultiLineString(..) => "MultiLineString",
            Value::Polygon(..) => "Polygon",
            Value::MultiPolygon(..) => "MultiPolygon",
            Value::GeometryCollection(..) => "GeometryCollection",
        });

        map.insert(String::from("type"), json_val(&ty));

        map.insert(String::from(match geometry.value {
            Value::GeometryCollection(..) => "geometries",
            _ => "coordinates",
        }), json_val(&geometry.value));
        return map;
    }
}

impl FromObject for Geometry {
    fn from_object(object: &JsonObject) -> Result<Self, Error> {
        let type_ = expect_type!(object);
        let value = match type_ {
            "Point" =>
                Value::Point(try!(util::get_coords_one_pos(object))),
            "MultiPoint" =>
                Value::MultiPoint(try!(util::get_coords_1d_pos(object))),
            "LineString" =>
                Value::LineString(try!(util::get_coords_1d_pos(object))),
            "MultiLineString" =>
                Value::MultiLineString(try!(util::get_coords_2d_pos(object))),
            "Polygon" =>
                Value::Polygon(try!(util::get_coords_2d_pos(object))),
            "MultiPolygon" =>
                Value::MultiPolygon(try!(util::get_coords_3d_pos(object))),
            "GeometryCollection" =>
                Value::GeometryCollection(try!(util::get_geometries(object))),
            _ => return Err(Error::GeometryUnknownType),
        };

        let bbox = try!(util::get_bbox(object));
        let crs = try!(util::get_crs(object));

        return Ok(Geometry {
            bbox: bbox,
            value: value,
            crs: crs,
        });
    }
}

#[cfg(not(feature = "with-serde"))]
impl ToJson for Geometry {
    fn to_json(&self) -> JsonValue {
        return ::rustc_serialize::json::Json::Object(self.into());
    }
}

#[cfg(feature = "with-serde")]
impl Serialize for Geometry {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer {
        JsonObject::from(self).serialize(serializer)
    }
}

#[cfg(feature = "with-serde")]
impl Deserialize for Geometry {
    fn deserialize<D>(deserializer: &mut D) -> Result<Geometry, D::Error>
    where D: Deserializer {
        use std::error::Error as StdError;

        let val = try!(JsonValue::deserialize(deserializer));

        if let Some(geo) = val.as_object() {
            Geometry::from_object(geo).map_err(|e| D::Error::custom(e.description()))
        }
        else {
            Err(D::Error::custom("expected json object"))
        }
    }
}


#[cfg(test)]
mod tests {
    use ::{GeoJson, Geometry, Value};

    #[cfg(not(feature = "with-serde"))]
    fn encode(geometry: &Geometry) -> String {
        use rustc_serialize::json::{self, ToJson};

        json::encode(&geometry.to_json()).unwrap()
    }
    #[cfg(feature = "with-serde")]
    fn encode(geometry: &Geometry) -> String {
        use serde_json;

        serde_json::to_string(&geometry).unwrap()
    }

    #[cfg(not(feature = "with-serde"))]
    fn decode(json_string: String) -> GeoJson {
        json_string.parse().unwrap()
    }
    #[cfg(feature = "with-serde")]
    fn decode(json_string: String) -> GeoJson {
        json_string.parse().unwrap()
    }

    #[test]
    fn encode_decode_geometry() {
        let geometry_json_str = "{\"coordinates\":[1.1,2.1],\"type\":\"Point\"}";
        let geometry = Geometry {
            value: Value::Point(vec![1.1, 2.1]),
            crs: None,
            bbox: None,
        };

        // Test encode
        let json_string = encode(&geometry);
        assert_eq!(json_string, geometry_json_str);

        // Test decode
        let decoded_geometry = match decode(json_string) {
            GeoJson::Geometry(g) => g,
            _ => unreachable!(),
        };
        assert_eq!(decoded_geometry, geometry);
    }
}
