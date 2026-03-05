# Changelog

## [0.6.0](https://github.com/xylex-group/supabase_rs/compare/v0.5.1...v0.6.0) (2026-03-05)


### Features

* add PostgREST RPC support with RpcBuilder ([690cef8](https://github.com/xylex-group/supabase_rs/commit/690cef8176f95f1f152d8acb5126e89d8be43bf7))
* Enhance code quality and maintainability ([46037d3](https://github.com/xylex-group/supabase_rs/commit/46037d38fcb1d288c0a22b34d216c99b0ebb3508))
* extend type generation for RPC functions ([2537cd0](https://github.com/xylex-group/supabase_rs/commit/2537cd04ce808798bf15e94065078a1a01407ec3))


### Bug Fixes

* **dependencies:** update reqwest to version 0.13.1 and correct rustls feature ([c44071b](https://github.com/xylex-group/supabase_rs/commit/c44071bbf9bab0bc9cdee7ae4d70aab0ed6b05e9))
* Fixes clippy and formatting warnings ([053f80d](https://github.com/xylex-group/supabase_rs/commit/053f80dc266764e03ce8703b301ad56c8c803df5))

## [0.5.1](https://github.com/floris-xlx/supabase_rs/compare/v0.5.0...v0.5.1) (2026-01-03)


### Bug Fixes

* Address clippy warnings across multiple modules ([12fecc9](https://github.com/floris-xlx/supabase_rs/commit/12fecc972a7d5faf01a6651cce0cff3b60ab0370))

## [0.4.0](https://github.com/floris-xlx/supabase_rs/compare/v0.3.5...v0.4.0) (2024-09-26)


### Features

* add upsert_without_defined_key method ([e632844](https://github.com/floris-xlx/supabase_rs/commit/e6328443168f0c427d03bb5c57c0a01d778a0c84))


### Bug Fixes

* return type of upsert_without_defined_key should be Result&lt;(), String&gt; ([eaee89b](https://github.com/floris-xlx/supabase_rs/commit/eaee89b43740f2b4bf5a8ae693669e9489de5ead))

## 0.1.0 (2024-07-10)


### Features

* add `bulk_insert` function ([33f30e3](https://github.com/floris-xlx/supabase_rs/commit/33f30e36d3e62e2884912b615b0140c2bf3542df))
* Add error message for table name not ending with Collection ([27dd186](https://github.com/floris-xlx/supabase_rs/commit/27dd1869d879663cb97f96f309c24f1d4440e1fe))
* add insert_without_defined_key method ([dc8e0ed](https://github.com/floris-xlx/supabase_rs/commit/dc8e0ed002ddc535e3d1f9bc0d851fb67336d38a))
* Add support for selecting specific columns in the select query ([309d8dc](https://github.com/floris-xlx/supabase_rs/commit/309d8dcb9c63259f48b801cb9400a84c7c6bfffe))
* allow users to select column names to reference when updating ([b2de629](https://github.com/floris-xlx/supabase_rs/commit/b2de62944a8f682b6a9d2664bcf2926617420919))
* implement missing Query methods ([4d0b9ba](https://github.com/floris-xlx/supabase_rs/commit/4d0b9ba13571063a0412d86030ea650fbaea2bf1))
* Query::build() takes filters and sort ([766592d](https://github.com/floris-xlx/supabase_rs/commit/766592dfd8097441dfdd3711f90d1bf62dfcc888))


### Bug Fixes

* abstracted all tests into seperate methods with files ([d2c60b8](https://github.com/floris-xlx/supabase_rs/commit/d2c60b8e24a37c11f3199f9358b813fe357b20f6))
* derive Default instead of manually implementing it ([77299ad](https://github.com/floris-xlx/supabase_rs/commit/77299ad9816917013c799b1aaac0c958a8149818))
* ignore or modify some doctests ([e6a2e03](https://github.com/floris-xlx/supabase_rs/commit/e6a2e0361e7e5001b57e57cf6158c3080a0492ab))
* implement std::fmt::Display for Filter instead of creating to_string() ([0bce354](https://github.com/floris-xlx/supabase_rs/commit/0bce35484e303c51404fe60bf597aeb7a46a295a))
* use `assert!()` instead of `assert_eq!()` for bool literal comparison ([efd10df](https://github.com/floris-xlx/supabase_rs/commit/efd10df39eb58b88d38f3966dcf8e2112a5ab1a8))
