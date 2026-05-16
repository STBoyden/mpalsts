#![cfg(test)]

use std::pin::pin;

use futures::StreamExt;

use super::*;

fn scaffold() {
	env_logger::try_init().ok();
}

#[test]
fn test_read() {
	smol::block_on(async {
		scaffold();

		if !is_sensor_available() {
			log::info!("No sensor available for this device");
			return;
		}

		let mut sensor = get_platform_reader()
			.await
			.expect("Device should have a sensor");

		let result = sensor.read().await;

		log::info!("[READ] Sensor output: {result:?}");
		assert!(result.is_ok(), "Sensor read should succeed");
	});
}

#[test]
fn test_poll_read() {
	smol::block_on(async {
		scaffold();

		if !is_sensor_available() {
			log::info!("No sensor available for this device");
			return;
		}

		let mut sensor = get_platform_reader()
			.await
			.expect("Device should have a sensor");

		let mut stream = pin!(sensor.stream(Duration::from_secs(1)));
		const MAX_ITERATIONS: i32 = 10;
		let mut iterations = 0;

		while iterations < MAX_ITERATIONS
			&& let Some(Ok(value)) = stream.next().await
		{
			log::info!("[POLL] Sensor output: {value:?}");
			iterations += 1;
		}

		assert!(
			iterations >= MAX_ITERATIONS,
			"Failed to poll sensor stream for {MAX_ITERATIONS} iterations: {iterations} iterations completed"
		);
	})
}

#[test]
fn test_mutate() {
	smol::block_on(async {
		scaffold();

		if !is_sensor_available() {
			log::info!("No sensor available for this device");
			return;
		}

		let sensor = get_platform_reader()
			.await
			.expect("Device should have a sensor");

		sensor.mutate_concrete(|mut concrete| {
			let before_mutate = concrete._test;
			let after_mutate = concrete.test_mutable();

			assert_ne!(
				before_mutate, after_mutate,
				"Sensor test mutable should change the _test value"
			);
		});
	});
}
