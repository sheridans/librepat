use librepat_core::ImporterRegistry;

pub(crate) fn registry() -> ImporterRegistry {
    let mut registry = ImporterRegistry::new();
    librepat_fluke::register(&mut registry);
    librepat_seaward::register(&mut registry);
    librepat_kewtech::register(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    #[test]
    fn application_registry_should_include_every_supported_adapter() {
        let format_ids = super::registry()
            .metadata()
            .map(|metadata| metadata.format_id)
            .collect::<Vec<_>>();
        assert_eq!(
            format_ids,
            [
                "fluke-flk-text",
                "seaward-primetest-ascii",
                "kewtech-kt74-77-ascii",
            ]
        );
    }
}
