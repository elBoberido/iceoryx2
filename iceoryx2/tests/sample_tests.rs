// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[generic_tests::define]
mod sample {
    use iceoryx2::port::publisher::Publisher;
    use iceoryx2::port::subscriber::Subscriber;
    use iceoryx2::prelude::*;
    use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    struct TestContext<Sut: Service> {
        service_name: ServiceName,
        service: PortFactory<Sut, u64>,
        publisher_1: Publisher<Sut, u64>,
        publisher_2: Publisher<Sut, u64>,
        subscriber: Subscriber<Sut, u64>,
    }

    impl<Sut: Service> TestContext<Sut> {
        fn new() -> Self {
            let service_name = generate_name();
            let service = Sut::new(&service_name)
                .publish_subscribe()
                .max_publishers(2)
                .max_subscribers(1)
                .typed::<u64>()
                .create()
                .unwrap();

            let publisher_1 = service.publisher().create().unwrap();

            let publisher_2 = service.publisher().create().unwrap();

            let subscriber = service.subscriber().create().unwrap();

            Self {
                service_name,
                service,
                publisher_1,
                publisher_2,
                subscriber,
            }
        }
    }

    #[test]
    fn origin_is_tracked_correctly<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();

        assert_that!(test_context.publisher_1.send_copy(123), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq test_context.publisher_1.id());

        assert_that!(test_context.publisher_2.send_copy(123), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(sample.origin(), eq test_context.publisher_2.id());
    }

    #[test]
    fn sample_of_dropped_service_does_not_block_new_service_creation<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();

        let service_name = test_context.service_name;

        assert_that!(test_context.publisher_1.send_copy(5), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap();
        assert_that!(sample, is_some);

        drop(test_context);

        assert_that!(
            Sut::new(&service_name)
                .publish_subscribe()
                .typed::<u64>()
                .create(),
            is_ok
        );
    }

    #[test]
    fn when_everything_is_dropped_the_sample_can_still_be_consumed<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();

        let sut = test_context.service;
        let publisher_1 = test_context.publisher_1;
        let publisher_2 = test_context.publisher_2;
        let subscriber = test_context.subscriber;

        drop(sut);

        const PAYLOAD: u64 = 8761238679123;

        assert_that!(publisher_1.send_copy(PAYLOAD), eq Ok(1));
        let sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);
        drop(publisher_1);
        drop(publisher_2);

        assert_that!(*sample, eq PAYLOAD);
    }

    #[test]
    fn sample_received_from_dropped_publisher_does_not_block_new_publishers<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();
        const PAYLOAD_1: u64 = 123554;

        let publisher = test_context.publisher_1;

        assert_that!(publisher.send_copy(PAYLOAD_1), eq Ok(1));
        let sample = test_context.subscriber.receive().unwrap().unwrap();

        drop(publisher);

        const PAYLOAD_2: u64 = 123981235645;

        let publisher = test_context.service.publisher().create().unwrap();
        assert_that!(publisher.send_copy(PAYLOAD_2), eq Ok(1));
        assert_that!(*sample, eq PAYLOAD_1);
        let sample_2 = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(*sample_2, eq PAYLOAD_2);
    }

    #[test]
    fn sample_from_dropped_subscriber_does_not_block_new_subscribers<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();
        const PAYLOAD_1: u64 = 7781123554;

        let subscriber = test_context.subscriber;

        assert_that!(test_context.publisher_1.send_copy(PAYLOAD_1), eq Ok(1));
        let _sample = subscriber.receive().unwrap().unwrap();

        drop(subscriber);

        const PAYLOAD_2: u64 = 123666645;

        let subscriber = test_context.service.subscriber().create().unwrap();
        assert_that!(test_context.publisher_1.send_copy(PAYLOAD_2), eq Ok(1));
        let sample_1 = subscriber.receive().unwrap().unwrap();
        let sample_2 = subscriber.receive().unwrap().unwrap();
        assert_that!(*sample_1, eq PAYLOAD_1);
        assert_that!(*sample_2, eq PAYLOAD_2);
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
