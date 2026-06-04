#![cfg(test)]

use std::pin::pin;

use dotenv::dotenv;
use futures::StreamExt;
use tracing::info;

use super::*;

fn scaffold() {
	dotenv().ok();
	_ = tracing_subscriber::fmt::try_init();
}

#[test]
fn test_read() {
	smol::block_on(async {
		scaffold();

		if !is_sensor_available() {
			info!("No sensor available for this device");
			return;
		}

		let sensor = get_platform_reader();

		let result = sensor.read().await;

		info!("[READ] Sensor output: {result:?}");
		assert!(result.is_ok(), "Sensor read should succeed");
	});
}

#[test]
fn test_poll_read() {
	smol::block_on(async {
		scaffold();

		if !is_sensor_available() {
			info!("No sensor available for this device");
			return;
		}

		let sensor = get_platform_reader();

		let mut stream = pin!(sensor.stream(Duration::from_secs(1)));
		const MAX_ITERATIONS: i32 = 10;
		let mut iterations = 0;

		while iterations < MAX_ITERATIONS
			&& let Some(Ok(value)) = stream.next().await
		{
			info!("[POLL] Sensor output: {value:?}");
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
			info!("No sensor available for this device");
			return;
		}

		let sensor = get_platform_reader();

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
