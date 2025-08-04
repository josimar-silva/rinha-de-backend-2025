# Goal

Process payments with a P99 of 100ms

# Optimizations

- https://deterministic.space/high-performance-rust.html
- https://deterministic.space/secret-life-of-cows.html
- https://likebike.com/posts/How_To_Write_Fast_Rust_Code.html
- http://troubles.md/posts/rustfest-2018-workshop/
- https://nnethercote.github.io/perf-book/build-configuration.html

# Performance Tests Results

| Test Run | Commit SHA                                                                                                        | Timestamp            | P99 (ms)            | Success Requests | Failed Requests | Lag  | Score | Flamegraph |
|----------|-------------------------------------------------------------------------------------------------------------------|----------------------|---------------------|------------------|-----------------|------|-------|------------|
| N/A      | [6eb13d6](https://github.com/josimar-silva/rinha-de-backend-2025/commit/6eb13d67e4905b88eeec17f9025b3fd72b1378b4) | 2025-07-25T13:53:29Z | 60.24655469999998ms | 7337             | 9551            | 7337 | 0     | N/A        |
| N/A      | [f6bac2f](https://github.com/josimar-silva/rinha-de-backend-2025/commit/f6bac2fce7bea700a0fc80da2eaca448187df9cf) | 2025-07-25T13:56:06Z | 1402.7065316ms      | 8441             | 8681            | 8441 | 0     | N/A        |
| N/A      | [5ac0e74](https://github.com/josimar-silva/rinha-de-backend-2025/commit/5ac0e7415a0b6b8f3f23ac7bcffe17a7287d7704) | 2025-07-28T04:07:33Z | 1117.29ms           | 8317             | 8792            | 8317 | 0     | N/A        |
| N/A      | [e4cfaca](https://github.com/josimar-silva/rinha-de-backend-2025/commit/e4cfacad7127c0c135f9990bb1eb4ff2ad944169) | 2025-07-28T07:30:03Z | 81.99ms             | 7342             | 9526            | 7342 | 0     | N/A        |
| N/A      | [4ca14c2](https://github.com/josimar-silva/rinha-de-backend-2025/commit/4ca14c2858883ad6a19774510d6cfee4e45886d8) | 2025-07-28T08:02:52Z | 74.23ms             | 7290             | 9541            | 7290 | 0     | N/A        |
| N/A      | [abc98cd](https://github.com/josimar-silva/rinha-de-backend-2025/commit/abc98cd7fbe850264836f55cc30ba5b092a37476) | 2025-07-28T17:13:22Z | 1364.99ms           | 7584             | 9245            | 7584 | 0     | N/A        |
| N/A      | [3473249](https://github.com/josimar-silva/rinha-de-backend-2025/commit/347324997764c428bd698710715f4b1b52f5180b) | 2025-07-28T17:29:17Z | 1242.5ms            | 8423             | 8738            | 8423 | 0     | N/A        |
| N/A      | [b460cb5](https://github.com/josimar-silva/rinha-de-backend-2025/commit/b460cb5b81f843b641c4752f7621ac692b06aa5f) | 2025-07-28T18:57:07Z | 1395.43ms           | 7627             | 9239            | 7627 | 0     | N/A        |
| N/A      | [9464402](https://github.com/josimar-silva/rinha-de-backend-2025/commit/9464402ad8aeb8e13a019c83776418420a162a81) | 2025-07-28T18:59:55Z | 1372.76ms           | 8114             | 8906            | 8114 | 0     | N/A        |
| N/A      | [f6b637a](https://github.com/josimar-silva/rinha-de-backend-2025/commit/f6b637ac4594cf1e08f2ec63f84c89d731e72286) | 2025-07-28T20:09:42Z | 62.45ms             | 7294             | 9562            | 7294 | 0     | N/A        |
| N/A      | [32cdeb9](https://github.com/josimar-silva/rinha-de-backend-2025/commit/32cdeb9a3013f7634200a12d252d1da6467f6bf8) | 2025-07-29T04:09:53Z | 1406.43ms           | 8264             | 8714            | 8264 | 0     | N/A        |
| N/A      | [abe997d](https://github.com/josimar-silva/rinha-de-backend-2025/commit/abe997dc223f1b93bfa010f5c91d00691fb831fe) | 2025-07-30T04:05:39Z | 1318ms              | 7780             | 9161            | 7780 | 0     | N/A        |
| N/A      | [7b77e4c](https://github.com/josimar-silva/rinha-de-backend-2025/commit/7b77e4c38d435ce30748c140cd5289f7c5a57c93) | 2025-07-31T04:04:49Z | 1312.38ms           | 8002             | 9021            | 8002 | 0     | N/A        |
| N/A      | [1591b81](https://github.com/josimar-silva/rinha-de-backend-2025/commit/1591b8134e320ff7ccd6486c587d29c086e23802) | 2025-08-01T04:16:07Z | 73.1ms              | 7304             | 9555            | 7304 | 0     | N/A        |
| N/A      | [0da19ab](https://github.com/josimar-silva/rinha-de-backend-2025/commit/0da19ab114026b83297dc7c84c06f99f0fb3e008) | 2025-08-01T08:07:58Z | 1397.67ms           | 8139             | 8831            | 8139 | 0     | N/A        |
| N/A      | [11bc22e](https://github.com/josimar-silva/rinha-de-backend-2025/commit/11bc22e3ce7964f76f1d88b166cf0efcee53a462) | 2025-08-01T14:31:35Z | 84.54ms             | 7251             | 9567            | 7251 | 0     | N/A        |
| N/A      | [110e86c](https://github.com/josimar-silva/rinha-de-backend-2025/commit/110e86cf5c1c1811e9421d8051bf36fee5a85420) | 2025-08-01T14:52:36Z | 1272.24ms           | 8416             | 8724            | 8416 | 0     | N/A        |
| N/A      | [816e9ce](https://github.com/josimar-silva/rinha-de-backend-2025/commit/816e9ce0f52028bf131e49236ff2a11ea7c405bf) | 2025-08-01T15:28:02Z | 1335.23ms           | 8119             | 8987            | 8119 | 0     | N/A        |
| N/A      | [bcd7241](https://github.com/josimar-silva/rinha-de-backend-2025/commit/bcd724190efbb55af38c7387fe5adf2cbbe067e6) | 2025-08-04T04:18:30Z | 81.85ms             | 7332             | 9542            | 7332 | 0     | N/A        |
