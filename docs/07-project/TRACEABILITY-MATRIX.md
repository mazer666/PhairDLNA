# Requirements traceability matrix

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / QA

Generated from the requirement definitions by
`scripts/generate_requirement_views.py`. Do not edit by hand.


**How to read this table.** Every requirement is traced to the feature that owns
it, the milestone that delivers it, the module area that will implement it, and the
verification that will prove it. The *evidence* column is filled only with output
that was actually observed; an empty cell means *not implemented*, not *passed*.


| ID | Feature | Milestone | Module area | Verification | Evidence | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `CFG-001` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-002` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-003` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-004` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-005` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-006` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-007` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-008` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | `crates/phairdlna-core/src/media/cache.rs::tests::the_cache_stays_inside_its_bound`, `crates/phairdlna-core/tests/artwork.rs::an_embedded_cover_that_is_too_large_is_refused` — the bound of the cache of the pictures is honoured and the oldest pictures are removed when it is exceeded; the limit of the concurrent streams, the limit of the extraction time and the number of the index workers are the work of M2.4 | Implemented |
| `CFG-009` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-010` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-011` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-012` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-013` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-014` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `CFG-015` | `F-12` | M2 | `crates/phairdlna-core/src/config` | Automated configuration test | — | Not implemented |
| `DOC-001` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-002` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-003` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-004` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-005` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-006` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-007` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-008` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-009` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-010` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-011` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `DOC-012` | `F-19` | M0 | `docs/**` | Documentation gate | — | Not implemented |
| `FUN-001` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-002` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-003` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-004` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-005` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-006` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-007` | `F-08` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-008` | `F-09` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-009` | `F-09` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-010` | `F-09` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-011` | `F-09` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-012` | `F-09` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-013` | `F-10` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/artwork.rs::an_embedded_cover_is_found`, `::an_embedded_cover_of_a_m4a_is_found`, `::a_sidecar_wins_over_the_embedded_cover`, `::a_sidecar_may_carry_the_name_of_the_file`, `::a_sidecar_outside_the_root_is_not_used`, `::a_track_without_a_picture_has_none`, `::a_file_outside_the_root_has_no_picture` — the cover of an ID3v2 tag and the cover of an MP4 container are read, the image next to the file wins over the cover in the tag, a sidecar that leads out of the root is refused, and a file without a picture names no address; the parsing happens in the server process until the sandbox child of ADR-0008 exists (M-06, ADR-0011) | Implemented |
| `FUN-014` | `F-10` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/src/media/thumbnail.rs::tests::the_produced_picture_is_a_jpeg`, `crates/phairdlna-core/src/media/cache.rs::tests::the_cache_stays_inside_its_bound`, `::a_deleted_directory_is_recreated`, `crates/phairdlna-cli/tests/artwork.rs::the_thumbnail_of_a_photo_is_answered`, `::the_second_request_is_answered_from_the_cache` — a photo is its own picture, the thumbnail is produced on demand and is kept in a disposable cache that stays inside its bound; the video poster, the second half of the requirement, is not produced, because the baseline ships no video decoder (F-10 § 12 Q1) | Implemented |
| `FUN-015` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-016` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-017` | `F-08` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/volume_events.rs::a_drive_that_is_unplugged_takes_its_files_with_it`, `::a_drive_that_comes_back_keeps_its_identifiers`, `crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_was_unmounted_takes_its_files_with_it`, `crates/phairdlna-cli/src/serve/watch.rs::tests::the_watch_reads_the_catalogue_again` — a drive that is unplugged takes its files with it, a drive that comes back is published again, and the running daemon notices both; the physical drive itself remains a manual device check | Implemented |
| `FUN-018` | `F-08` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/media_roots.rs::the_scanner_never_writes_to_the_library` — the walk opens directories and files to read them and writes nothing; the cache directory is not configured yet, so the exception of the requirement does not apply | Implemented |
| `FUN-019` | `F-08` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/media_roots_multiple.rs::the_path_of_a_file_is_the_path_of_the_disk` — the path of the catalogue is the path of the disk, byte for byte; the identity is a separate, derived value | Verified |
| `FUN-020` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-021` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Protocol conformance + device test | — | Not implemented |
| `FUN-022` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Protocol conformance + device test | — | Not implemented |
| `FUN-023` | `F-08` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-024` | `F-08` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-025` | `F-09` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-026` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-027` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-028` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-029` | `F-11` | M2 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-040` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/http_media.rs::the_client_receives_the_bytes_of_the_file` — the bytes come from the file, never from a conversion | Verified |
| `FUN-041` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | `crates/phairdlna-core/tests/http_media.rs::the_type_of_the_answer_is_the_type_of_the_file` — the codec is part of the type and of the DLNA profile token | Implemented |
| `FUN-042` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-043` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-044` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-045` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-046` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-047` | `F-17` | M5 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-048` | `F-17` | M5 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-049` | `F-17` | M5 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `FUN-050` | `F-06` | M1 | `crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media` | Automated test | — | Not implemented |
| `INT-001` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`, `crates/phairdlna-core/src/ssdp/request.rs::tests::the_reply_carries_the_required_headers` — the daemon answers a real search with the headers of the specification; the deadline of one second is not measured, the answer follows the `MX` window of INT-008 | Implemented |
| `INT-002` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-cli/tests/discovery.rs::an_unknown_search_is_ignored`, `crates/phairdlna-core/src/ssdp/targets.rs::tests::a_search_for_all_answers_with_all` — a search for an advertised target is answered, a search for anything else is not | Verified |
| `INT-003` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered` — the answer arrives on the socket of the client that sent the search | Verified |
| `INT-004` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`, `::the_advertisements_are_withdrawn` — the daemon sends the `ssdp:alive` advertisements when it starts and the `ssdp:byebye` when it stops; the interval between two cycles (`max-age/4`) is implemented but not observed by a test | Implemented |
| `INT-005` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`, `crates/phairdlna-core/src/ssdp/targets.rs::tests::five_targets_are_advertised` — the `USN` of the answer carries the form of the specification | Verified |
| `INT-006` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered` — the answer names the description of the server; the address of the interface the request arrived on is not chosen, the daemon has one address | Implemented |
| `INT-007` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered` — the daemon binds the IPv4 group and answers on it; the IPv6 link-local group is not implemented | Implemented |
| `INT-008` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/ssdp/request.rs::tests::a_search_without_mx_is_ignored`, `::tests::an_out_of_range_mx_is_ignored`, `::tests::the_reply_delay_stays_inside_the_window` | Verified |
| `INT-009` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-010` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | `crates/phairdlna-core/src/ssdp/request.rs::tests::another_method_is_ignored`, `::tests::a_wrong_man_is_ignored` — a datagram the parser does not accept is dropped without a log record | Verified |
| `INT-011` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-012` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform`, `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered` — the product identity travels in `SERVER` and the `UDN` in `USN`; the friendly name, the manufacturer and the model name travel in the device description, not in the SSDP headers | Implemented |
| `INT-013` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/ssdp/request.rs::tests::a_wrong_man_is_ignored` — a search whose `MAN` header is not the quoted token is not answered | Verified |
| `INT-014` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/identity.rs::tests::the_identity_survives_a_restart` | Verified |
| `INT-015` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-016` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-cli/src/serve/ssdp.rs` — the daemon sets the time to live of the advertisements to 2; the value is not observed on the wire | Implemented |
| `INT-017` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/ssdp/targets.rs::tests::five_targets_are_advertised`, `::tests::both_services_are_advertised` | Verified |
| `INT-018` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-019` | `F-01` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform` | Verified |
| `INT-020` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_root_device_carries_its_identity`, `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered` — the document is generated and the daemon serves it over HTTP | Verified |
| `INT-021` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services`, `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered` — the description names the service documents and the control endpoints, and the daemon serves them | Verified |
| `INT-022` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::every_service_entry_carries_its_addresses`, `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered` — the URLs are absolute and the daemon serves every one of them at the address it advertises | Verified |
| `INT-023` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/scpd.rs::every_state_variable_declares_its_data_type`, `crates/phairdlna-cli/tests/serve.rs::the_service_descriptions_are_answered` — the service descriptions are generated and the daemon serves both of them | Verified |
| `INT-024` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_document_is_well_formed` | Verified |
| `INT-025` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/soap_negative.rs::an_unknown_action_is_the_fault_401` | Verified |
| `INT-026` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-027` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::a_name_with_reserved_characters_is_escaped` | Verified |
| `INT-028` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_content_directory_is_advertised_in_version_one`, `crates/phairdlna-cli/tests/serve.rs::the_service_descriptions_are_answered` — the service is advertised and the daemon serves its description | Verified |
| `INT-029` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_connection_manager_is_advertised_without_transcoding`, `crates/phairdlna-cli/tests/serve.rs::the_connection_manager_is_answered` — the service is advertised and its control endpoint answers | Verified |
| `INT-030` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_browse_answer_carries_the_four_arguments_in_order` | Verified |
| `INT-031` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_browse_answer_carries_the_four_arguments_in_order` | Verified |
| `INT-032` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::a_browse_with_a_limit_returns_at_most_that_many` | Verified |
| `INT-033` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::a_metadata_browse_describes_the_container` | Verified |
| `INT-034` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_filter_narrows_the_answer_without_failing` | Verified |
| `INT-035` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory_search.rs::the_capabilities_name_the_properties_the_server_uses` | Verified |
| `INT-036` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory_search.rs::a_search_finds_the_matching_item` | Verified |
| `INT-037` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/catalogue_index.rs::the_update_id_never_goes_backwards`, `::the_update_id_changes_when_a_file_changes` — the identifier of the real catalogue never decreases and changes when the catalogue changes; it starts again after a restart, because the counter that keeps it monotonic is not written to the disk (FUN-009) | Implemented |
| `INT-038` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_root_and_the_content_directory_root_are_known`, `crates/phairdlna-core/tests/catalogue_index.rs::the_root_container_has_the_parent_id_minus_one` — the catalogue of the library answers with the identifiers 0 and 1 | Implemented |
| `INT-039` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::an_unknown_object_is_the_fault_701` | Verified |
| `INT-040` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_write_actions_are_absent_and_not_answered` | Verified |
| `INT-041` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::a_browse_with_an_offset_returns_the_rest` | Verified |
| `INT-042` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::the_result_is_a_didl_lite_document_that_parses_on_its_own`, `crates/phairdlna-core/tests/catalogue_index.rs::a_browse_of_a_real_folder_shows_its_files` — the writer of F-05 answers with the real catalogue of F-08 | Implemented |
| `INT-043` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::a_browse_of_the_root_reports_the_system_update_identifier` | Verified |
| `INT-044` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services` | Verified |
| `INT-045` | `F-02` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/device_description.rs::the_document_holds_exactly_one_device_element` | Verified |
| `INT-046` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::an_uncountable_container_answers_with_zero_or_720` | Verified |
| `INT-047` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory_search.rs::the_sort_criteria_order_the_answer` | Verified |
| `INT-048` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory.rs::no_non_standard_action_is_answered` | Verified |
| `INT-049` | `F-03` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/content_directory_search.rs::the_capabilities_name_the_properties_the_server_uses` | Verified |
| `INT-050` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::the_source_list_has_four_fields_per_entry` | Verified |
| `INT-051` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::the_source_list_holds_every_format_of_the_registry` | Verified |
| `INT-052` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::the_list_of_the_running_transfers_is_empty` | Verified |
| `INT-053` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::a_preparation_creates_no_transfer` — no connection is ever opened; the transfer that binds a connection to a media object needs the HTTP layer | Implemented |
| `INT-054` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-055` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::every_entry_carries_the_wildcard_and_names_no_profile` | Verified |
| `INT-056` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/connection_manager.rs::no_entry_uses_the_push_protocol` | Verified |
| `INT-057` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-058` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-059` | `F-04` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-060` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for` — the attributes are written; the client that reads them is not tested yet | Implemented |
| `INT-061` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_classes_are_the_ones_of_the_schema` | Verified |
| `INT-062` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_identifier_is_stable_and_carries_no_path` — the identifier of the fixture is opaque; the hash of ARCHITECTURE §8 is F-08 | Implemented |
| `INT-063` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema` | Verified |
| `INT-064` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_protocol_names_a_format_the_server_serves` | Verified |
| `INT-065` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_escaping.rs::the_reserved_characters_are_escaped_in_the_text` | Verified |
| `INT-066` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for` | Verified |
| `INT-067` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_artwork_address_carries_no_dlna_attribute`, `crates/phairdlna-cli/tests/artwork.rs::the_address_of_the_cover_is_absolute_and_names_no_profile`, `::the_browse_of_a_track_without_a_picture_names_no_address` — the address is absolute, names no path of the disk and carries no `dlna:profileID`; the endpoint that serves it is `/art/<id>`, and an object without a picture names no address at all | Implemented |
| `INT-068` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags` | Verified |
| `INT-069` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags` | Verified |
| `INT-070` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags` | Verified |
| `INT-071` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for` | Verified |
| `INT-072` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::a_very_long_title_is_cut_and_marked` | Verified |
| `INT-073` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_escaping.rs::the_document_survives_the_second_escaping` | Verified |
| `INT-074` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema` — the attribute carries the number the catalogue read; the unit is the catalogue's contract | Implemented |
| `INT-075` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema` | Verified |
| `INT-076` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags` | Verified |
| `INT-077` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_document_never_carries_a_write_status` | Verified |
| `INT-078` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::the_two_modes_describe_the_same_file` — the mode is chosen by the dispatcher; the client profile that selects it is F-07 | Implemented |
| `INT-079` | `F-05` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/didl_resource.rs::nothing_is_claimed_that_the_server_cannot_prove` — the tokens are written as the requirement asks; the conformance check of a profile does not exist yet, so the catalogue claims no profile | Implemented |
| `INT-080` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::a_get_answers_the_whole_file` | Verified |
| `INT-081` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::a_closed_range_is_answered_with_206` | Verified |
| `INT-082` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::a_range_behind_the_file_is_answered_with_416` | Verified |
| `INT-083` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::a_broken_range_is_answered_with_416` | Verified |
| `INT-084` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::a_get_answers_the_whole_file` | Verified |
| `INT-085` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::a_set_of_ranges_is_answered_as_one_range` | Verified |
| `INT-086` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::a_head_answers_the_headers_without_a_body` | Verified |
| `INT-087` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::a_file_without_a_type_is_sent_as_octet_stream` — the type comes from the catalogue, which hands it over already refined; the sniffing is F-10 | Implemented |
| `INT-088` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::the_answer_carries_a_length_and_no_chunked_encoding` | Verified |
| `INT-089` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::the_answer_names_the_file_to_read` — the transfer slot and the file descriptor belong to the runtime (F-13, F-18) | Implemented |
| `INT-090` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning` | Verified |
| `INT-091` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning` — reopening a file that was removed and restored is the file layer's (F-08, F-13) | Implemented |
| `INT-092` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::a_range_on_a_head_is_ignored` | Verified |
| `INT-093` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_range.rs::an_if_range_that_does_not_hold_ignores_the_range` | Verified |
| `INT-094` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_negative.rs::an_unknown_object_is_answered_with_404` | Verified |
| `INT-095` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_media.rs::a_connection_is_kept_alive_unless_it_asks_to_close` | Verified |
| `INT-096` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_dlna_headers.rs::a_transfer_mode_the_server_does_not_offer_is_answered_with_406` | Verified |
| `INT-097` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_dlna_headers.rs::another_value_for_the_content_features_is_answered_with_400` | Verified |
| `INT-098` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_path_containment.rs::a_traversal_attempt_is_answered_with_404`, `crates/phairdlna-cli/tests/serve.rs::a_file_that_left_the_root_is_not_served` — the daemon resolves the path again before it opens the file and refuses one that left the root | Verified |
| `INT-099` | `F-06` | M1 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | `crates/phairdlna-core/tests/http_dlna_headers.rs::a_media_answer_carries_the_dlna_headers` | Verified |
| `INT-100` | `F-07` | M4 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | — | Not implemented |
| `INT-101` | `F-07` | M4 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-102` | `F-07` | M4 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | — | Not implemented |
| `INT-103` | `F-07` | M4 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance + device test | — | Not implemented |
| `INT-104` | `F-07` | M4 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `INT-105` | `F-11` | M2 | `crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl` | Protocol conformance test | — | Not implemented |
| `LIFE-001` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-002` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-003` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-004` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-005` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-006` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-007` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-008` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-009` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-010` | `F-15` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `LIFE-011` | `F-13` | M2 | `crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli` | Lifecycle and recovery test | — | Not implemented |
| `MB-001` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-002` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-003` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-004` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-005` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-006` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-007` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-008` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `MB-009` | `F-15` | M2 | `crates/phairdlna-mac, docs/05-quality` | Manual macOS usability protocol | — | Not implemented |
| `OPS-001` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | `crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client` — the command starts the daemon in the foreground, and the daemon answers a client on the network | Verified |
| `OPS-002` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-003` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-004` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-005` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-006` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | `crates/phairdlna-cli/src/serve/log.rs::tests::a_known_moment_is_read_correctly`, `::tests::the_level_has_a_name` — a record carries a timestamp, a level, a component and a message with named fields | Verified |
| `OPS-007` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-008` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | `crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`, `crates/phairdlna-cli/tests/discovery.rs::the_advertisements_are_withdrawn` — a `SIGTERM` makes the daemon withdraw its advertisements, stop the discovery, drain the HTTP server and exit with the code 0; the flush of the index does not exist, because the index on disk (`FUN-009`) does not exist | Implemented |
| `OPS-009` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-010` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-011` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-012` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | `crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform` | Verified |
| `OPS-013` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-014` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-015` | `F-13` | M1 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | `crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client` — the daemon serves on a port above 1024 and needs no privilege | Verified |
| `OPS-016` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-017` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-020` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-021` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-022` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-023` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-024` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-025` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-026` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `OPS-027` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-admin` | Automated CLI test | — | Not implemented |
| `PERF-001` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-002` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-003` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-004` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-005` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-006` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-007` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-008` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-009` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-010` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-011` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-012` | `F-09` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-013` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-014` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PERF-015` | `F-18` | M1 | `crates/phairdlna-core/src/** (benchmarked)` | Benchmark | — | Not implemented |
| `PLT-001` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-002` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-003` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-004` | `F-15` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-005` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-006` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-007` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-008` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-009` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-010` | `F-22` | M0 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-011` | `F-22` | M2 | `crates/phairdlna-core/src/platform` | Build matrix | `crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_was_unmounted_takes_its_files_with_it`, `::a_volume_that_was_mounted_again_is_published_again`, `crates/phairdlna-core/tests/volume_events.rs::a_drive_that_comes_back_under_another_identity_is_given_new_identifiers` — a mount, an unmount and a remount are first-class events, and the entries of a volume that comes back under another identity are given new identifiers; the core honours the events of the platform through `Catalogue::apply_volume_event`, and the daemon derives the same outcome from the observations of the roots, because the portable adapter has no native events — the native watcher (DiskArbitration) and the check on the build matrix are the work of the platform adapter (F-13) | Implemented |
| `PLT-012` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-013` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-014` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-015` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-016` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-017` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PLT-018` | `F-22` | M3 | `crates/phairdlna-core/src/platform` | Build matrix | — | Not implemented |
| `PRD-001` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-002` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-003` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-004` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-005` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-006` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-007` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-008` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-009` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-010` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-011` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-012` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-013` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-014` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `PRD-015` | `cross-cutting` | M0 | `cross-cutting (see docs/01-product/FEATURE-CATALOG.md)` | Manual acceptance protocol | — | Not implemented |
| `REL-001` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `REL-002` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `REL-003` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `REL-004` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `REL-005` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `REL-006` | `F-18` | M4 | `crates/phairdlna-core/tests/soak, crates/phairdlna-cli` | Fault-injection and soak test | — | Not implemented |
| `RES-001` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-002` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-003` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-004` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-005` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-006` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-007` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-008` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `RES-009` | `F-18` | M2 | `crates/phairdlna-core/src/scheduler` | Resource-pressure test | — | Not implemented |
| `SEC-001` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-002` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-003` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-004` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-005` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-006` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-007` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-008` | `F-08` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-010` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | `crates/phairdlna-core/tests/xml_reader.rs::a_document_type_declaration_is_refused` | Verified |
| `SEC-011` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-012` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-013` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-014` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-015` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-016` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-017` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-018` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-060` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | `crates/phairdlna-core/tests/xml_reader.rs::a_document_that_is_too_large_is_refused` | Verified |
| `SEC-061` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-062` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-063` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-064` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-065` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-066` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-067` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | `crates/phairdlna-core/src/media/thumbnail.rs::tests::a_picture_that_lies_about_its_size_is_refused_before_it_is_decoded`, `::a_picture_with_too_many_pixels_is_refused` — the declared size of a picture is read from its header and checked before a single pixel is allocated; the decoding happens in the server process until the sandbox child of ADR-0008 exists (M-06, ADR-0011) | Implemented |
| `SEC-068` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-069` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-070` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-071` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-072` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-090` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-091` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-092` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-093` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-094` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-095` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SEC-096` | `F-16` | M1 | `crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, crates/phairdlna-core/src/upnp` | Negative security test | — | Not implemented |
| `SUP-001` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-002` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-003` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-004` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-005` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-006` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-007` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-008` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `SUP-009` | `F-19` | M0 | `Cargo.toml, deny.toml, .github/workflows` | Licence and advisory gate | — | Not implemented |
| `UX-001` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-002` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-003` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-004` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-005` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-006` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-007` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-008` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-009` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-010` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-011` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-012` | `F-14` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-013` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-014` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-015` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-016` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-017` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-018` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-019` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |
| `UX-020` | `F-15` | M2 | `crates/phairdlna-cli, crates/phairdlna-mac` | Manual usability protocol | — | Not implemented |

---

## How evidence is recorded

When a requirement is implemented, the *Evidence* cell receives:

- an automated test: `cargo test -p phairdlna-core catalogue::tests::…` (test name),
- a gate: the gate ID and the observed output, or
- a device record: the row in `../05-quality/INTEROPERABILITY-MATRIX.md`.

A cell is only filled with output that was actually observed. A skipped check is
written as `NOT RUN — <reason>` and never as a pass.
