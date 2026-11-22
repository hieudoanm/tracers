# Framework fixtures

Minimal repositories used by `crates/tracers-analysis/tests/framework_tests.rs`
to verify framework detection. Each subdirectory is grouped by language:

```
fixtures/frameworks/<language>/<framework>/
```

| Group        | Frameworks                                                         |
| ------------ | ------------------------------------------------------------------ |
| `typescript` | react, vue, angular, nestjs, nextjs, hono                          |
| `javascript` | express                                                            |
| `python`     | django, flask, fastapi, celery, click, typer, pydantic, sqlalchemy |
| `go`         | gin, echo, cobra                                                   |
| `rust`       | actix-web, axum, clap                                              |
| `java`       | spring-boot, jakarta-ee                                            |
| `kotlin`     | ktor                                                               |
| `csharp`     | aspnetcore                                                         |
| `php`        | laravel, symfony                                                   |
| `ruby`       | rails                                                              |

Group names match `fixtures/languages/<language>/` where one exists;
`javascript` covers plain `.js`/`.jsx` fixtures.
