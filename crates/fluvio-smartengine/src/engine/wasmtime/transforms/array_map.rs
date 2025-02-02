#[cfg(test)]
mod test {

    use std::{convert::TryFrom};

    use fluvio_smartmodule::{
        dataplane::smartmodule::{SmartModuleInput},
        Record,
    };

    use crate::engine::{
        SmartEngine, SmartModuleChainBuilder, SmartModuleConfig, metrics::SmartModuleChainMetrics,
        wasmtime::transforms::simple_transform::ARRAY_MAP_FN_NAME,
    };

    const SM_ARRAY_MAP: &str = "fluvio_smartmodule_array_map_array";

    use crate::engine::fixture::read_wasm_module;

    #[ignore]
    #[test]
    fn test_array_map() {
        let engine = SmartEngine::new();
        let mut chain_builder = SmartModuleChainBuilder::default();

        chain_builder.add_smart_module(
            SmartModuleConfig::builder().build().unwrap(),
            read_wasm_module(SM_ARRAY_MAP),
        );

        let mut chain = chain_builder
            .initialize(&engine)
            .expect("failed to build chain");

        assert_eq!(
            chain.instances().first().expect("first").transform().name(),
            ARRAY_MAP_FN_NAME
        );

        let metrics = SmartModuleChainMetrics::default();

        let input = vec![Record::new("[\"Apple\",\"Banana\",\"Cranberry\"]")];
        let output = chain
            .process(SmartModuleInput::try_from(input).expect("input"), &metrics)
            .expect("process");
        assert_eq!(output.successes.len(), 3); // generate 3 records
        assert_eq!(output.successes[0].value.as_ref(), b"\"Apple\"");
        assert_eq!(output.successes[1].value.as_ref(), b"\"Banana\"");
        assert_eq!(output.successes[2].value.as_ref(), b"\"Cranberry\"");
    }
}
