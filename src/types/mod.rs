pub(crate) mod klipper;

pub(crate) trait MetricsExporter {
    fn describe(&self) {}
    fn export(&self, name: Option<&String>);
}
