pub(crate) mod klipper;
pub(crate) mod moonraker;

pub(crate) trait MetricsExporter {
    fn describe(&self) {}
    fn export(&self, _name: Option<&String>) {}
}
