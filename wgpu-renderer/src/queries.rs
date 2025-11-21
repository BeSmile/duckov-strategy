
// https://github.com/gfx-rs/wgpu/blob/119b4efada475f95507f8f577bf1abfe3d529fd0/examples/features/src/timestamp_queries/mod.rs

struct Queries {
    set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    destination_buffer: wgpu::Buffer,
    num_queries: u64,
    next_unused_query: u32,
}

struct QueryResults {
    encoder_timestamps: [u64; 2],
    render_start_end_timestamps: [u64; 2],
    render_inside_timestamp: Option<u64>,
    compute_start_end_timestamps: [u64; 2],
    compute_inside_timestamp: Option<u64>,
}

impl QueryResults {
    // Queries:
    // * encoder timestamp start
    // * encoder timestamp end
    // * render start
    // * render in-between (optional)
    // * render end
    // * compute start
    // * compute in-between (optional)
    // * compute end
    const NUM_QUERIES: u64 = 8;

    #[expect(
        clippy::redundant_closure,
        reason = "false positive for `get_next_slot`, which needs to be used by reference"
    )]
    fn from_raw_results(timestamps: Vec<u64>, timestamps_inside_passes: bool) -> Self {
        assert_eq!(timestamps.len(), Self::NUM_QUERIES as usize);

        let mut next_slot = 0;
        let mut get_next_slot = || {
            let slot = timestamps[next_slot];
            next_slot += 1;
            slot
        };

        let mut encoder_timestamps = [0, 0];
        encoder_timestamps[0] = get_next_slot();
        let render_start_end_timestamps = [get_next_slot(), get_next_slot()];
        let render_inside_timestamp = timestamps_inside_passes.then(|| get_next_slot());
        let compute_start_end_timestamps = [get_next_slot(), get_next_slot()];
        let compute_inside_timestamp = timestamps_inside_passes.then(|| get_next_slot());
        encoder_timestamps[1] = get_next_slot();

        QueryResults {
            encoder_timestamps,
            render_start_end_timestamps,
            render_inside_timestamp,
            compute_start_end_timestamps,
            compute_inside_timestamp,
        }
    }

    fn print(&self, queue: &wgpu::Queue) {
        let period = queue.get_timestamp_period();
        let elapsed_us = |start, end: u64| end.wrapping_sub(start) as f64 * period as f64 / 1000.0;

        println!(
            "Elapsed time before render until after compute: {:.2} μs",
            elapsed_us(self.encoder_timestamps[0], self.encoder_timestamps[1]),
        );
        println!(
            "Elapsed time render pass: {:.2} μs",
            elapsed_us(
                self.render_start_end_timestamps[0],
                self.render_start_end_timestamps[1]
            )
        );
        if let Some(timestamp) = self.render_inside_timestamp {
            println!(
                "Elapsed time first triangle: {:.2} μs",
                elapsed_us(self.render_start_end_timestamps[0], timestamp)
            );
        }
        println!(
            "Elapsed time compute pass: {:.2} μs",
            elapsed_us(
                self.compute_start_end_timestamps[0],
                self.compute_start_end_timestamps[1]
            )
        );
        if let Some(timestamp) = self.compute_inside_timestamp {
            println!(
                "Elapsed time after first dispatch: {:.2} μs",
                elapsed_us(self.compute_start_end_timestamps[0], timestamp)
            );
        }
    }
}

impl Queries {
    fn new(device: &wgpu::Device, num_queries: u64) -> Self {
        Queries {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: num_queries as _,
                ty: wgpu::QueryType::Timestamp,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query resolve buffer"),
                size: size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            }),
            destination_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query dest buffer"),
                size: size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            num_queries,
            next_unused_query: 0,
        }
    }

    fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(
            &self.set,
            // TODO(https://github.com/gfx-rs/wgpu/issues/3993): Musn't be larger than the number valid queries in the set.
            0..self.next_unused_query,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.destination_buffer,
            0,
            self.resolve_buffer.size(),
        );
    }

    fn wait_for_results(&self, device: &wgpu::Device, is_test_on_metal: bool) -> Vec<u64> {
        self.destination_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| ());
        let poll_type = if is_test_on_metal {
            // Use a short timeout because the `timestamps_encoder` test (which
            // is also marked as flaky) has been observed to hang on Metal.
            //
            // Note that a timeout here is *not* considered an error. In this
            // particular case that is what we want, but in general, waits in
            // tests should probably treat a timeout as an error.
            wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(std::time::Duration::from_secs(5)),
            }
        } else {
            wgpu::PollType::wait_indefinitely()
        };
        device.poll(poll_type).unwrap();

        let timestamps = {
            let timestamp_view = self
                .destination_buffer
                .slice(..(size_of::<u64>() as wgpu::BufferAddress * self.num_queries))
                .get_mapped_range();
            bytemuck::cast_slice(&timestamp_view).to_vec()
        };

        self.destination_buffer.unmap();

        timestamps
    }
}