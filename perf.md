# Goal

Process payments with a P99 of 100ms

# Optimizations

- https://deterministic.space/high-performance-rust.html
- https://deterministic.space/secret-life-of-cows.html
- https://likebike.com/posts/How_To_Write_Fast_Rust_Code.html
- http://troubles.md/posts/rustfest-2018-workshop/
- https://nnethercote.github.io/perf-book/build-configuration.html

# Flamegraph Analysis

## [#107](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16713859094/job/47303559607)

Finally, got the [flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16713859094/artifacts/3678343125) collection working. Yay!!!

Overall, there is quite a bit of CPU time being consumed by the PaymentQueue operations.
A good percentage of the time is being used to acquire the multiplexed connection to Redis.

This has a huge impact on the Payments API handler, which is propagating the requests directly to Redis.
Being locked waiting for the connection means more latency time.  
Let's see how it performs after offloading the call to Redis to a MPSC channel. 

### Backend 01
![flamegraph-backend-01.svg](docs/flamegraphs/107/flamegraph-backend-01.svg)

### Backend 02
![flamegraph-backend-02.svg](docs/flamegraphs/107/flamegraph-backend-02.svg)

## [#108](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16727183039)

As theorized, moving the [Payment push to a channel](https://github.com/josimar-silva/rinha-de-backend-2025/pull/20) removed the contention on the request handler. 

As shown on the performance test results bellow, all requests were successfully processed this time.

Although the requests were all processed, the payments weren't hence the `Lag` of 22423, which the diff between the requests received and the ones processed.

Let's see how we can decrease the lag next.

### Backend 01
![flamegraph-backend-01.svg](docs/flamegraphs/108/flamegraph-backend-01.svg)

### Backend 02
![flamegraph-backend-02.svg](docs/flamegraphs/108/flamegraph-backend-02.svg)

## [#109](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16729766895)

On [c5c68fc](https://github.com/josimar-silva/rinha-de-backend-2025/commit/c5c68fc2e6599bd15fbbb3ed0b1d07e65ade473f), we introduce
multiple instances of the `payment_processing_worker` to increase the throughput of payment processing.
The change had no effect on the `lag`. The bottleneck points to the Redis connection acquiring logic.

Instead of reusing the multiplexed connection, PaymentQueue and PaymentRepository, we are acquiring a new connection from
the client on every operation, which is quite inefficient.
In hindsight, this was a big oversight during the implementation 🤦🏽. But hey, that's why we have automated performance tests 😌.

### Backend 01
![flamegraph-backend-01.svg](docs/flamegraphs/109/flamegraph-backend-01.svg)

### Backend 02
![flamegraph-backend-02.svg](docs/flamegraphs/109/flamegraph-backend-02.svg)

## [#111](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16756984103)

With [#19](https://github.com/josimar-silva/rinha-de-backend-2025/pull/19), we introduced a more efficient 
way of handling multiplexed connections from Redis and safely (using [Arc](https://doc.rust-lang.org/book/ch16-03-shared-state.html?highlight=Arc#atomic-reference-counting-with-arct)) shared.

This change drastically reduced the `lag`, as can be seen in the test results.

There's a bit of contention with Redis connection, let's see if we can improve it even more and reduce the `lag` to **0**.

### Backend 01
![flamegraph-backend-01.svg](docs/flamegraphs/111/flamegraph-backend-01.svg)

### Backend 02
![flamegraph-backend-02.svg](docs/flamegraphs/111/flamegraph-backend-02.svg)


# Performance Tests Results

| Test Run                                                                                | Commit SHA                                                                                                        | Timestamp            | Max. Requests | P99 (ms)            | Success Requests | Failed Requests | Lag   | Score              | Flamegraph                                                                                                         |
|-----------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|----------------------|---------------|---------------------|------------------|-----------------|-------|--------------------|--------------------------------------------------------------------------------------------------------------------|
| [#87](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16559770057)  | [6eb13d6](https://github.com/josimar-silva/rinha-de-backend-2025/commit/6eb13d67e4905b88eeec17f9025b3fd72b1378b4) | 2025-07-25T13:53:29Z | 1000          | 60.24655469999998ms | 7337             | 9551            | 7337  | 0                  | N/A                                                                                                                |
| [#88](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16562894079)  | [f6bac2f](https://github.com/josimar-silva/rinha-de-backend-2025/commit/f6bac2fce7bea700a0fc80da2eaca448187df9cf) | 2025-07-25T13:56:06Z | 1000          | 1402.7065316ms      | 8441             | 8681            | 8441  | 0                  | N/A                                                                                                                |
| [#89](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16563537280)  | [5ac0e74](https://github.com/josimar-silva/rinha-de-backend-2025/commit/5ac0e7415a0b6b8f3f23ac7bcffe17a7287d7704) | 2025-07-28T04:07:33Z | 1000          | 1117.29ms           | 8317             | 8792            | 8317  | 0                  | N/A                                                                                                                |
| [#91](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16575538619)  | [e4cfaca](https://github.com/josimar-silva/rinha-de-backend-2025/commit/e4cfacad7127c0c135f9990bb1eb4ff2ad944169) | 2025-07-28T07:30:03Z | 1000          | 81.99ms             | 7342             | 9526            | 7342  | 0                  | N/A                                                                                                                |
| [#92](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16575864980)  | [4ca14c2](https://github.com/josimar-silva/rinha-de-backend-2025/commit/4ca14c2858883ad6a19774510d6cfee4e45886d8) | 2025-07-28T08:02:52Z | 1000          | 74.23ms             | 7290             | 9541            | 7290  | 0                  | N/A                                                                                                                |
| [#93](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16577706611)  | [abc98cd](https://github.com/josimar-silva/rinha-de-backend-2025/commit/abc98cd7fbe850264836f55cc30ba5b092a37476) | 2025-07-28T17:13:22Z | 1000          | 1364.99ms           | 7584             | 9245            | 7584  | 0                  | N/A                                                                                                                |
| [#94](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16577763153)  | [3473249](https://github.com/josimar-silva/rinha-de-backend-2025/commit/347324997764c428bd698710715f4b1b52f5180b) | 2025-07-28T17:29:17Z | 1000          | 1242.5ms            | 8423             | 8738            | 8423  | 0                  | N/A                                                                                                                |
| [#95](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16578734465)  | [b460cb5](https://github.com/josimar-silva/rinha-de-backend-2025/commit/b460cb5b81f843b641c4752f7621ac692b06aa5f) | 2025-07-28T18:57:07Z | 1000          | 1395.43ms           | 7627             | 9239            | 7627  | 0                  | N/A                                                                                                                |
| [#96](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16579171194)  | [9464402](https://github.com/josimar-silva/rinha-de-backend-2025/commit/9464402ad8aeb8e13a019c83776418420a162a81) | 2025-07-28T18:59:55Z | 1000          | 1372.76ms           | 8114             | 8906            | 8114  | 0                  | N/A                                                                                                                |
| [#97](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16586425395)  | [f6b637a](https://github.com/josimar-silva/rinha-de-backend-2025/commit/f6b637ac4594cf1e08f2ec63f84c89d731e72286) | 2025-07-28T20:09:42Z | 1000          | 62.45ms             | 7294             | 9562            | 7294  | 0                  | N/A                                                                                                                |
| [#98](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16613167118)  | [32cdeb9](https://github.com/josimar-silva/rinha-de-backend-2025/commit/32cdeb9a3013f7634200a12d252d1da6467f6bf8) | 2025-07-29T04:09:53Z | 1000          | 1406.43ms           | 8264             | 8714            | 8264  | 0                  | N/A                                                                                                                |
| [#99](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16639527158)  | [abe997d](https://github.com/josimar-silva/rinha-de-backend-2025/commit/abe997dc223f1b93bfa010f5c91d00691fb831fe) | 2025-07-30T04:05:39Z | 1000          | 1318ms              | 7780             | 9161            | 7780  | 0                  | N/A                                                                                                                |
| [#100](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16666121730) | [7b77e4c](https://github.com/josimar-silva/rinha-de-backend-2025/commit/7b77e4c38d435ce30748c140cd5289f7c5a57c93) | 2025-07-31T04:04:49Z | 1000          | 1312.38ms           | 8002             | 9021            | 8002  | 0                  | N/A                                                                                                                |
| [#101](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16669843009) | [1591b81](https://github.com/josimar-silva/rinha-de-backend-2025/commit/1591b8134e320ff7ccd6486c587d29c086e23802) | 2025-08-01T04:16:07Z | 1000          | 73.1ms              | 7304             | 9555            | 7304  | 0                  | N/A                                                                                                                |
| [#102](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16672986226) | [0da19ab](https://github.com/josimar-silva/rinha-de-backend-2025/commit/0da19ab114026b83297dc7c84c06f99f0fb3e008) | 2025-08-01T08:07:58Z | 1000          | 1397.67ms           | 8139             | 8831            | 8139  | 0                  | N/A                                                                                                                |
| [#103](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16677680995) | [11bc22e](https://github.com/josimar-silva/rinha-de-backend-2025/commit/11bc22e3ce7964f76f1d88b166cf0efcee53a462) | 2025-08-01T14:31:35Z | 1000          | 84.54ms             | 7251             | 9567            | 7251  | 0                  | N/A                                                                                                                |
| [#105](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16678138338) | [110e86c](https://github.com/josimar-silva/rinha-de-backend-2025/commit/110e86cf5c1c1811e9421d8051bf36fee5a85420) | 2025-08-01T14:52:36Z | 1000          | 1272.24ms           | 8416             | 8724            | 8416  | 0                  | N/A                                                                                                                |
| [#106](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16678910803) | [816e9ce](https://github.com/josimar-silva/rinha-de-backend-2025/commit/816e9ce0f52028bf131e49236ff2a11ea7c405bf) | 2025-08-01T15:28:02Z | 1000          | 1335.23ms           | 8119             | 8987            | 8119  | 0                  | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16678910803/artifacts/3668455362) |
| [#107](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16713859094) | [bcd7241](https://github.com/josimar-silva/rinha-de-backend-2025/commit/bcd724190efbb55af38c7387fe5adf2cbbe067e6) | 2025-08-04T04:18:30Z | 1000          | 81.85ms             | 7332             | 9542            | 7332  | 0                  | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16713859094/artifacts/3678343125) |
| [#108](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16727183039) | [a8b13cc](https://github.com/josimar-silva/rinha-de-backend-2025/commit/a8b13cca38f66c2e38ef0e954cf143cbae0b2e34) | 2025-08-04T15:20:26Z | 1000          | 49.73ms             | 30231            | 0               | 22423 | 135610.53999999244 | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16727183039/artifacts/3682892137) |
| [#109](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16729766895) | [c5c68fc](https://github.com/josimar-silva/rinha-de-backend-2025/commit/c5c68fc2e6599bd15fbbb3ed0b1d07e65ade473f) | 2025-08-04T17:22:33Z | 1000          | 49ms                | 30359            | 0               | 22528 | 137159.7549999934  | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16729766895/artifacts/3683929073) |
| [#110](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16740294991) | [b6abb71](https://github.com/josimar-silva/rinha-de-backend-2025/commit/b6abb71bcd6bc8893106773e737241660e7b5e2c) | 2025-08-05T04:09:59Z | 1000          | 51.32ms             | 30229            | 0               | 21983 | 144911.79999999434 | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16740294991/artifacts/3687513186) |
| [#111](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16756984103) | [203d983](https://github.com/josimar-silva/rinha-de-backend-2025/commit/203d983da66ec5c548d827aa4cdb0e6689fae8bf) | 2025-08-05T17:34:45Z | 1000          | 3.23ms              | 30448            | 0               | 419   | 645279.7482002105  | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16756984103/artifacts/3693475527) |
| [#112](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16767258714) | [230eb14](https://github.com/josimar-silva/rinha-de-backend-2025/commit/230eb14c86c9cc1c7b28f32888622ec2fe6ed2c6) | 2025-08-06T04:07:13Z | 1000          | 3.99ms              | 30439            | 0               | 272   | 636443.2527002069  | [Flamegraph](https://github.com/josimar-silva/rinha-de-backend-2025/actions/runs/16767258714/artifacts/3697231226) |
