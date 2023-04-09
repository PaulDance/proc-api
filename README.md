# `proc-api`: REST API to a host's processes
## Context

This is a solution to an exercise that was given to me as an assignment during
a recruitment process. As it revealed itself to be not so trivial and quite
interesting in terms of async and synchronization, I am publishing it here for
others to see.


## API

It mostly revolves around a system of cache storing basic information about the
processes currently running on the OS hosting the server:

 * `POST /acquire_process_list`:
   * Refreshes the internal process cache and returns an empty body.
   * The response's status code can be either `200 OK` or
     `500 INTERNAL SERVER ERROR`, depending on the success of the operation.

 * `GET /processes`:
   * Fetches the cached information and returns it as a JSON array of simple
     objects matching the following example format:

     ```json
     [
       {
         "pid": 1,
         "uid": 0,
         "name": "systemd",
         "username": "root"
       },
       ...
     ]
     ```

   * If `POST /acquire_process_list` has not been requested previously, the
     cache is thus empty, the response is therefore `[]`.
   * If it is called multiple times without any `POST /acquire_process_list`
     in between, the same response is given back every time.

 * `GET /search?pid=<pid>&uid=<uid>&name=<name>&username=<username>`:
   * In effect, it enables filtering the results given by `GET /processes`.
   * All data attributes made available through `GET /processes` can be used as
     query URL parameters order to filter the results.
   * If two or more parameters are present, then the result is the intersection
     of the results of each filter, that is to say the filter tests are AND-ed.
   * All parameters are optional. However, if all are absent, the request is
     rejected on a `400 BAD REQUEST`.

 * `GET /data`:
   * A Server-Sent Events (SSE) endpoint enabling to stream newly-collected
     processes as data events.
   * Events are always data events, with each event body following the same
     format previously described for `GET /processes`.
   * Upon opening the stream, all currently-cached processes are immediately
     emitted as data events. It thereby achieves what `GET /processes` does,
     but in an SSE fashion.
   * With an opened stream, whenever `POST /acquire_process_list` is called in
     parallel, only the new processes observed since the last refresh are
     returned through the stream, concurrently to the server's normal operation.


## Usage
### Installation

 * Clone the [current repository](https://github.com/PaulDance/proc-api).
 * Run it with: `cargo run`. Some CLI options are available, see: `--help`.
 * The server is then made available at `http://127.0.0.1:8080` by default.

### Testing

Some basic integration tests are included: run `cargo test` to check them.

### Documentation

Unit documentation is included in the modules: run `cargo doc
--document-private-items` to check it out localy with a browser.


## Architecture

The project is based on:
 * Warp for the async HTTP server.
 * Tokio for the async runner.
 * Serde and `serde_json` for the JSON manipulation.
 * Sysinfo for the actual process collection.

Files:
 * `src/main.rs`: just launches the server. Also contains integration tests.
 * `src/proc.rs`: implements the process representation, collection and caching.
 * `src/routes.rs`: routes defining the acceptable requests using Warp filters.
 * `src/handlers.rs`: async functions handling the requests accepted and parsed
   by the routes.
