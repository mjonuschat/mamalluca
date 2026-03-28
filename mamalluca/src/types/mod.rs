pub(crate) mod klipper;
pub(crate) mod moonraker;

pub(crate) trait MetricsExporter {
    #[allow(dead_code)]
    fn describe(&self) {}
    fn export(&self, _name: Option<&String>) {}
}
