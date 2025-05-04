use slint::{ModelRc, SharedString, VecModel};
use std::collections::HashSet;
use std::rc::Rc;

pub fn scan_wifi() -> ModelRc<SharedString> {
    let wifi_list: Vec<String> = {
        let unique_wifi_list: HashSet<String> = wifiscanner::scan()
            .unwrap()
            .iter()
            .map(|wifi| wifi.ssid.clone())
            .filter(|ssid| !ssid.is_empty()) // Filter out empty strings
            .collect();

        unique_wifi_list.into_iter().collect()
    };

    let wifi_list_model: VecModel<SharedString> = VecModel::from(
        wifi_list
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<SharedString>>(),
    );
    ModelRc::from(Rc::new(wifi_list_model))
}
