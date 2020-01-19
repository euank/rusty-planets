use ::serde::Deserialize;

pub fn get_planetdata() -> Vec<PlanetData> {
    serde_json::from_str(data).unwrap()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanetData {
    pub id: u32,
    pub distance_from_sun: f64,
    pub orbital_velocity: f64,
    pub mass: f64,
    pub diameter: f64,
}

const data: &'static str = include_str!("../data/planets.json");
