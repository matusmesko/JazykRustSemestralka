use actix_web::web::ServiceConfig;

pub struct ControllerRegistry {
    pub name: &'static str,
    pub configure: fn(&mut ServiceConfig),
}

inventory::collect!(ControllerRegistry);


pub fn configure_all(cfg: &mut ServiceConfig) {
    for controller in inventory::iter::<ControllerRegistry> {
        (controller.configure)(cfg);
    }
}

