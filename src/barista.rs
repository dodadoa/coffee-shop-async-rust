use actix_web::web;
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub type Avaialble = bool;
pub type BaristaId = i32;
pub type Barista = HashMap<BaristaId, Avaialble>;

pub async fn init_baristas() -> Result<Barista> {
    let baristas: HashMap<BaristaId, Avaialble> =
        vec![0, 1, 2, 3, 4].into_iter().map(|i| (i, true)).collect();

    Ok(baristas)
}

pub async fn retrieve_available_barista_id(
    baristas: web::Data<Arc<Mutex<Barista>>>,
) -> Option<BaristaId> {
    let mut mutex_barista = baristas.lock().await;
    let available = mutex_barista
        .clone()
        .into_iter()
        .find_map(|(i, available)| if available { Some(i) } else { None });
    if available.is_some() {
        *mutex_barista.get_mut(&available.unwrap()).unwrap() = false
    }
    available
}

pub async fn free_barista_id(baristas: web::Data<Arc<Mutex<Barista>>>, available_id: &BaristaId) {
    let mut mutex_barista = baristas.lock().await;
    *mutex_barista.get_mut(available_id).unwrap() = true
}
