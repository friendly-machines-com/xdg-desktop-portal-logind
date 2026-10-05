;;; Guix package definition for xdg-desktop-portal-logind.
;;; Build with:   guix build -f guix1.scm
;;; Install with: guix package -f guix1.scm

(use-modules (guix packages)
             (guix git-download)
             (guix build-system cargo)
             (guix build utils)
             (guix gexp)
             ((guix licenses) #:prefix license:))

(define %source-directory
  (dirname (current-filename)))

(define %git-file?
  (git-predicate %source-directory))

(define rust-async-broadcast-0.7.2
  (crate-source "async-broadcast" "0.7.2"
                "0ckmqcwyqwbl2cijk1y4r0vy60i89gqc86ijrxzz5f2m4yjqfnj3"))

(define rust-async-channel-2.5.0
  (crate-source "async-channel" "2.5.0"
                "1ljq24ig8lgs2555myrrjighycpx2mbjgrm3q7lpa6rdsmnxjklj"))

(define rust-async-executor-1.14.0
  (crate-source "async-executor" "1.14.0"
                "0al1rmxjy7p7r6h50z698q5lwssqs5a2vzmqbazm1z2sv1rgjsy9"))

(define rust-async-io-2.6.0
  (crate-source "async-io" "2.6.0"
                "1z16s18bm4jxlmp6rif38mvn55442yd3wjvdfhvx4hkgxf7qlss5"))

(define rust-async-lock-3.4.2
  (crate-source "async-lock" "3.4.2"
                "04c3xrrdrfrvh9v0ajxrangpy38qi76qq268zslphnxxjqjpy3r9"))

(define rust-async-process-2.5.0
  (crate-source "async-process" "2.5.0"
                "0xfswxmng6835hjlfhv7k0jrfp7czqxpfj6y2s5dsp05q0g94l7w"))

(define rust-async-recursion-1.2.0
  (crate-source "async-recursion" "1.2.0"
                "0lg4v61ax9wnfb5b5m11895qddcmq5a6h57cihf6n9mdp89br2jg"))

(define rust-async-signal-0.2.14
  (crate-source "async-signal" "0.2.14"
                "11dlpb15la279r5cazppy18gbk2xzzl60ahzl19m1kr0l2psmdaj"))

(define rust-async-task-4.7.1
  (crate-source "async-task" "4.7.1"
                "1pp3avr4ri2nbh7s6y9ws0397nkx1zymmcr14sq761ljarh3axcb"))

(define rust-async-trait-0.1.92
  (crate-source "async-trait" "0.1.92"
                "0rqn5iga1hlv2lm8xzav1zhar46jb4dvx89i6kfv93kb53maxxl2"))

(define rust-atomic-waker-1.1.2
  (crate-source "atomic-waker" "1.1.2"
                "1h5av1lw56m0jf0fd3bchxq8a30xv0b4wv8s4zkp4s0i7mfvs18m"))

(define rust-autocfg-1.5.1
  (crate-source "autocfg" "1.5.1"
                "0lqasy5i30flcgih1b50kvsk6z32g09r1q4ql7q81pj6228jy0zj"))

(define rust-bitflags-2.13.2
  (crate-source "bitflags" "2.13.2"
                "01hbgjwvid66850fzi76mvn5f2bqycx6sf165ng1kfjqq9bl1v9x"))

(define rust-blocking-1.7.0
  (crate-source "blocking" "1.7.0"
                "1ykd0gj18r4v4b8r692hds5dsg2w6y9fq4nlxs2l7fbcvwll63m7"))

(define rust-bumpalo-3.20.3
  (crate-source "bumpalo" "3.20.3"
                "0jc6va3nwcqikm7chnpdv1s87my3gs2j7g1sc7g3k91brg3arxbj"))

(define rust-cfg-if-1.0.5
  (crate-source "cfg-if" "1.0.5"
                "0026j56901nzjraap3da0a8njw42j66zcxnn6s2s9aa5bcblhxjf"))

(define rust-concurrent-queue-2.5.0
  (crate-source "concurrent-queue" "2.5.0"
                "0wrr3mzq2ijdkxwndhf79k952cp4zkz35ray8hvsxl96xrx1k82c"))

(define rust-crossbeam-utils-0.8.23
  (crate-source "crossbeam-utils" "0.8.23"
                "1ilan2nw7fvka8hki80fr57a5dgd4mvcsvwq60437j6yvlwyw7m3"))

(define rust-endi-1.1.1
  (crate-source "endi" "1.1.1"
                "16a0076dx41vgrzzimm9clcym77h732czqjiajanmzvd1i1y5dv6"))

(define rust-enumflags2-0.7.12
  (crate-source "enumflags2" "0.7.12"
                "1vzcskg4dca2jiflsfx1p9yw1fvgzcakcs7cpip0agl51ilgf9qh"))

(define rust-enumflags2-derive-0.7.12
  (crate-source "enumflags2_derive" "0.7.12"
                "09rqffacafl1b83ir55hrah9gza0x7pzjn6lr6jm76fzix6qmiv7"))

(define rust-equivalent-1.0.2
  (crate-source "equivalent" "1.0.2"
                "03swzqznragy8n0x31lqc78g2af054jwivp7lkrbrc0khz74lyl7"))

(define rust-errno-0.3.14
  (crate-source "errno" "0.3.14"
                "1szgccmh8vgryqyadg8xd58mnwwicf39zmin3bsn63df2wbbgjir"))

(define rust-event-listener-5.4.2
  (crate-source "event-listener" "5.4.2"
                "1lk9sv7r07l58jk263s18896l55mx9jv0g1rm4hj2mpi3paas8ss"))

(define rust-event-listener-strategy-0.5.4
  (crate-source "event-listener-strategy" "0.5.4"
                "14rv18av8s7n8yixg38bxp5vg2qs394rl1w052by5npzmbgz7scb"))

(define rust-fastrand-2.5.0
  (crate-source "fastrand" "2.5.0"
                "08q2r30y62winysimnlpbvw9kiwn0rmdlidqlmzd6z90mv764z6s"))

(define rust-futures-core-0.3.34
  (crate-source "futures-core" "0.3.34"
                "0pjgv4fx0np6hrs5sz5a2phabwv0z70yr51v03injbi44bjrkmlj"))

(define rust-futures-io-0.3.34
  (crate-source "futures-io" "0.3.34"
                "1v9z6wj92ra18kpv0xig21hgpzrvcwmcr8fszyzh64yyay0zmh2k"))

(define rust-futures-lite-2.6.1
  (crate-source "futures-lite" "2.6.1"
                "1ba4dg26sc168vf60b1a23dv1d8rcf3v3ykz2psb7q70kxh113pp"))

(define rust-futures-task-0.3.34
  (crate-source "futures-task" "0.3.34"
                "1zfilqs8nwlfqz4prk7ihvpp5avvzins87ibzlxzq5fhs7ipshfd"))

(define rust-futures-util-0.3.34
  (crate-source "futures-util" "0.3.34"
                "1g3r9ghzq7c2fh34lis43i72xavk9p84npgfwgb5vfpqcwjajl0d"))

(define rust-getrandom-0.4.3
  (crate-source "getrandom" "0.4.3"
                "16b0202fkdwz3p2cyll82dv24ljbn0wiyy829v4lwbkbflyqh3ih"))

(define rust-hashbrown-0.17.1
  (crate-source "hashbrown" "0.17.1"
                "0jmqz7i4yl6cm7rbn0i2ffkfrmwi6xkmzkaldr2v8bcsx2v0jngd"))

(define rust-hermit-abi-0.5.3
  (crate-source "hermit-abi" "0.5.3"
                "115jzi6ixx2nhkzbr2ijj36634agz32n6ilz2rg7vk5s1vb94xg1"))

(define rust-hex-0.4.3
  (crate-source "hex" "0.4.3"
                "0w1a4davm1lgzpamwnba907aysmlrnygbqmfis2mqjx5m552a93z"))

(define rust-indexmap-2.14.2
  (crate-source "indexmap" "2.14.2"
                "0mf86hbjkkcd82cpq683bblbs0zwa8ndla96ci8p1ji6bl7ijknc"))

(define rust-js-sys-0.3.106
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "js-sys" "0.3.106"
                "1icwmpjw54lb7vg5926k5y4y5jbiih0zxhwgjwnzn475v90xk0vq"))

(define rust-libc-0.2.190
  (crate-source "libc" "0.2.190"
                "0y5yap4bfp7rfsldcbk9pb5alcgygca5xn1n2pmh181zdpf3spff"))

(define rust-linux-raw-sys-0.12.1
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "linux-raw-sys" "0.12.1"
                "0lwasljrqxjjfk9l2j8lyib1babh2qjlnhylqzl01nihw14nk9ij"))

(define rust-memchr-2.8.3
  (crate-source "memchr" "2.8.3"
                "161xa63ipfanf8v3nb82xd5hqgydv55nzw59wyngqbz6alfaz2yg"))

(define rust-memoffset-0.9.1
  (crate-source "memoffset" "0.9.1"
                "12i17wh9a9plx869g7j4whf62xw68k5zd4k0k5nh6ys5mszid028"))

(define rust-once-cell-1.21.4
  (crate-source "once_cell" "1.21.4"
                "0l1v676wf71kjg2khch4dphwh1jp3291ffiymr2mvy1kxd5kwz4z"))

(define rust-ordered-stream-0.2.0
  (crate-source "ordered-stream" "0.2.0"
                "0l0xxp697q7wiix1gnfn66xsss7fdhfivl2k7bvpjs4i3lgb18ls"))

(define rust-parking-2.2.1
  (crate-source "parking" "2.2.1"
                "1fnfgmzkfpjd69v4j9x737b1k8pnn054bvzcn5dm3pkgq595d3gk"))

(define rust-pin-project-lite-0.2.17
  (crate-source "pin-project-lite" "0.2.17"
                "1kfmwvs271si96zay4mm8887v5khw0c27jc9srw1a75ykvgj54x8"))

(define rust-piper-0.2.5
  (crate-source "piper" "0.2.5"
                "1hd3j94mw5dwc457gs9ssb2r5b9iipywndf5srqx7pj38jd4fdf8"))

(define rust-polling-3.11.0
  (crate-source "polling" "3.11.0"
                "0622qfbxi3gb0ly2c99n3xawp878fkrd1sl83hjdhisx11cly3jx"))

(define rust-proc-macro-crate-3.5.0
  (crate-source "proc-macro-crate" "3.5.0"
                "0kv1g1d1zjwxlgcaba2qlshzyy32j03xic8rskqlcr5mnblsfyz6"))

(define rust-proc-macro2-1.0.107
  (crate-source "proc-macro2" "1.0.107"
                "1nb6ly8kp65f724kj73ippc7lvydss24sm2vagk6qpklpg4pwplq"))

(define rust-quote-1.0.47
  (crate-source "quote" "1.0.47"
                "00ch0yyzvv6s671ik0kcsbw8nigdaj2g3fr61kcahwx48aqlvgqz"))

(define rust-r-efi-6.0.0
  (crate-source "r-efi" "6.0.0"
                "1gyrl2k5fyzj9k7kchg2n296z5881lg7070msabid09asp3wkp7q"))

(define rust-rustix-1.1.5
  (crate-source "rustix" "1.1.5"
                "17b2srw7rcqmrs1shj89g8i3r1447lihv7qrbxvp11j1psxgl7l9"))

(define rust-rustversion-1.0.23
  (crate-source "rustversion" "1.0.23"
                "07z2a843fs80fawwflj9jwn49k9b0bd0dhhbvy0ar69vaxd72m6g"))

(define rust-serde-1.0.229
  (crate-source "serde" "1.0.229"
                "1fp04fq4a79bpm61xz1zy0pbz4kpc7d771zii1k3inmszq55jj21"))

(define rust-serde-core-1.0.229
  (crate-source "serde_core" "1.0.229"
                "0j1ajiha76h3nmd976il9li6975k121xa7jb39ws8n0yqp4s5p37"))

(define rust-serde-derive-1.0.229
  (crate-source "serde_derive" "1.0.229"
                "0j4k63i7h1bikxwz2c89ig0hrwbnl9mz1czn85xx99x5cc9dg9g7"))

(define rust-serde-repr-0.1.21
  (crate-source "serde_repr" "0.1.21"
                "01l987ghc17h1y9cf9xbzmcs77575mbrjf4ca2h70g15vqlicfwd"))

(define rust-signal-hook-registry-1.4.8
  (crate-source "signal-hook-registry" "1.4.8"
                "06vc7pmnki6lmxar3z31gkyg9cw7py5x9g7px70gy2hil75nkny4"))

(define rust-slab-0.4.12
  (crate-source "slab" "0.4.12"
                "1xcwik6s6zbd3lf51kkrcicdq2j4c1fw0yjdai2apy9467i0sy8c"))

(define rust-syn-2.0.119
  (crate-source "syn" "2.0.119"
                "15vjy620l91a3q4n4f4gzhnflmdr6pnm38v2m6cpk86i8av32a47"))

(define rust-syn-3.0.6
  (crate-source "syn" "3.0.6"
                "1vmw7s58rzrs926nv5m06x7qbgswm1aa9iw3s1bj5var47kyi4w5"))

(define rust-tempfile-3.27.0
  (crate-source "tempfile" "3.27.0"
                "1gblhnyfjsbg9wjg194n89wrzah7jy3yzgnyzhp56f3v9jd7wj9j"))

(define rust-toml-datetime-1.1.1+spec-1.1.0
  (crate-source "toml_datetime" "1.1.1+spec-1.1.0"
                "1mws2mkkf46l7inn77azhm0vdwxngv9vsbhbl0ah33p2c9gzcr9i"))

(define rust-toml-edit-0.25.15+spec-1.1.0
  (crate-source "toml_edit" "0.25.15+spec-1.1.0"
                "0556lgzcvgfy16b8sxskr391s6cbfwnb0r4h5i4k6qw5lnaflh0k"))

(define rust-toml-parser-1.1.3+spec-1.1.0
  (crate-source "toml_parser" "1.1.3+spec-1.1.0"
                "0mjdvihdkmjd4ykh574xgii71hpxw7ns7h4n4bisqpxrz4faqf0x"))

(define rust-tracing-0.1.44
  (crate-source "tracing" "0.1.44"
                "006ilqkg1lmfdh3xhg3z762izfwmxcvz0w7m4qx2qajbz9i1drv3"))

(define rust-tracing-attributes-0.1.31
  (crate-source "tracing-attributes" "0.1.31"
                "1np8d77shfvz0n7camx2bsf1qw0zg331lra0hxb4cdwnxjjwz43l"))

(define rust-tracing-core-0.1.36
  (crate-source "tracing-core" "0.1.36"
                "16mpbz6p8vd6j7sf925k9k8wzvm9vdfsjbynbmaxxyq6v7wwm5yv"))

(define rust-uds-windows-1.2.1
  (crate-source "uds_windows" "1.2.1"
                "0vidqwwfgn8wyzvbxiqil787b4wyqjia50zpdbbjqx7n8wlgpxpj"))

(define rust-unicode-ident-1.0.26
  (crate-source "unicode-ident" "1.0.26"
                "0m3915ipi4zz7isncf5k1dz47ys0nq9j7l4l2n2rm03zaxwg8ifj"))

(define rust-uuid-1.27.0
  (crate-source "uuid" "1.27.0"
                "16h5h6bf5ybh1lj97lcdl41g7fqbiczpavpsb0zf3b63p4v7s9wp"))

(define rust-wasm-bindgen-0.2.129
  (crate-source "wasm-bindgen" "0.2.129"
                "02flhqld01jqb6vbfyx1gb8s78g1pnq2164daxad93y6mhrlzdcv"))

(define rust-wasm-bindgen-macro-0.2.129
  (crate-source "wasm-bindgen-macro" "0.2.129"
                "0dc5xq09sy1v9ns1fhb0cnyqz5p1wcjjvkdmxskj9qhnbg1x0a9f"))

(define rust-wasm-bindgen-macro-support-0.2.129
  (crate-source "wasm-bindgen-macro-support" "0.2.129"
                "1dx6w90f14avmhri7ss9lyz03060g6475r4axy3bm7biqf5ill3g"))

(define rust-wasm-bindgen-shared-0.2.129
  (crate-source "wasm-bindgen-shared" "0.2.129"
                "1ilmp5d3sl8lrq9djhcs90gk66yvk8mgwk4sfrvpvkd75b2wkw13"))

(define rust-windows-link-0.2.1
  (crate-source "windows-link" "0.2.1"
                "1rag186yfr3xx7piv5rg8b6im2dwcf8zldiflvb22xbzwli5507h"))

(define rust-windows-sys-0.61.2
  ;; TODO REVIEW: Check bundled sources.
  (crate-source "windows-sys" "0.61.2"
                "1z7k3y9b6b5h52kid57lvmvm05362zv1v8w0gc7xyv5xphlp44xf"))

(define rust-winnow-1.0.4
  (crate-source "winnow" "1.0.4"
                "10fzxipa7lx16172p3aca9j60hzbqgjki2f95kqksd5qywcp7f93"))

(define rust-zbus-5.19.0
  (crate-source "zbus" "5.19.0"
                "01sram5sgwsg3x8mghx77cjbsfa2c10mar7fnzj23d2w0xybxd2x"))

(define rust-zbus-macros-5.19.0
  (crate-source "zbus_macros" "5.19.0"
                "0h4gr26kyhdyn503rgg8h44sjxm8d6n8qbzpd0cdzrmd15fn7419"))

(define rust-zbus-names-4.3.4
  (crate-source "zbus_names" "4.3.4"
                "0kk250s3x1fxpz9fvhdr64ydbacpn8ah23hy021yhlzzlfs8igyq"))

(define rust-zcheapstr-1.1.0
  (crate-source "zcheapstr" "1.1.0"
                "0wwlv70bi2rydvvzfq249q6i51mjx85c4m2wxcx1hra5c18yrbyi"))

(define rust-zvariant-5.15.0
  (crate-source "zvariant" "5.15.0"
                "0iwihslxshfhalihp6kv7xz7nbv1p3b9sl97hi2izpbcrhklrly1"))

(define rust-zvariant-derive-5.15.0
  (crate-source "zvariant_derive" "5.15.0"
                "15y4z1rkcpvrz7dv7j2rfv8wiq6i8nzifj9pgw6dnlj3kgk5ahc6"))

(define rust-zvariant-utils-4.2.0
  (crate-source "zvariant_utils" "4.2.0"
                "18q80094ci64myzvcp0g2l3c6mnx7b3hsii8lfabc853c51jkl5s"))

(define-cargo-inputs lookup-cargo-inputs
                     (xdg-desktop-portal-logind =>
                                                (list
                                                 rust-async-broadcast-0.7.2
                                                 rust-async-channel-2.5.0
                                                 rust-async-executor-1.14.0
                                                 rust-async-io-2.6.0
                                                 rust-async-lock-3.4.2
                                                 rust-async-process-2.5.0
                                                 rust-async-recursion-1.2.0
                                                 rust-async-signal-0.2.14
                                                 rust-async-task-4.7.1
                                                 rust-async-trait-0.1.92
                                                 rust-atomic-waker-1.1.2
                                                 rust-autocfg-1.5.1
                                                 rust-bitflags-2.13.2
                                                 rust-blocking-1.7.0
                                                 rust-bumpalo-3.20.3
                                                 rust-cfg-if-1.0.5
                                                 rust-concurrent-queue-2.5.0
                                                 rust-crossbeam-utils-0.8.23
                                                 rust-endi-1.1.1
                                                 rust-enumflags2-0.7.12
                                                 rust-enumflags2-derive-0.7.12
                                                 rust-equivalent-1.0.2
                                                 rust-errno-0.3.14
                                                 rust-event-listener-5.4.2
                                                 rust-event-listener-strategy-0.5.4
                                                 rust-fastrand-2.5.0
                                                 rust-futures-core-0.3.34
                                                 rust-futures-io-0.3.34
                                                 rust-futures-lite-2.6.1
                                                 rust-futures-task-0.3.34
                                                 rust-futures-util-0.3.34
                                                 rust-getrandom-0.4.3
                                                 rust-hashbrown-0.17.1
                                                 rust-hermit-abi-0.5.3
                                                 rust-hex-0.4.3
                                                 rust-indexmap-2.14.2
                                                 rust-js-sys-0.3.106
                                                 rust-libc-0.2.190
                                                 rust-linux-raw-sys-0.12.1
                                                 rust-memchr-2.8.3
                                                 rust-memoffset-0.9.1
                                                 rust-once-cell-1.21.4
                                                 rust-ordered-stream-0.2.0
                                                 rust-parking-2.2.1
                                                 rust-pin-project-lite-0.2.17
                                                 rust-piper-0.2.5
                                                 rust-polling-3.11.0
                                                 rust-proc-macro-crate-3.5.0
                                                 rust-proc-macro2-1.0.107
                                                 rust-quote-1.0.47
                                                 rust-r-efi-6.0.0
                                                 rust-rustix-1.1.5
                                                 rust-rustversion-1.0.23
                                                 rust-serde-1.0.229
                                                 rust-serde-core-1.0.229
                                                 rust-serde-derive-1.0.229
                                                 rust-serde-repr-0.1.21
                                                 rust-signal-hook-registry-1.4.8
                                                 rust-slab-0.4.12
                                                 rust-syn-2.0.119
                                                 rust-syn-3.0.6
                                                 rust-tempfile-3.27.0
                                                 rust-toml-datetime-1.1.1+spec-1.1.0
                                                 rust-toml-edit-0.25.15+spec-1.1.0
                                                 rust-toml-parser-1.1.3+spec-1.1.0
                                                 rust-tracing-0.1.44
                                                 rust-tracing-attributes-0.1.31
                                                 rust-tracing-core-0.1.36
                                                 rust-uds-windows-1.2.1
                                                 rust-unicode-ident-1.0.26
                                                 rust-uuid-1.27.0
                                                 rust-wasm-bindgen-0.2.129
                                                 rust-wasm-bindgen-macro-0.2.129
                                                 rust-wasm-bindgen-macro-support-0.2.129
                                                 rust-wasm-bindgen-shared-0.2.129
                                                 rust-windows-link-0.2.1
                                                 rust-windows-sys-0.61.2
                                                 rust-winnow-1.0.4
                                                 rust-zbus-5.19.0
                                                 rust-zbus-macros-5.19.0
                                                 rust-zbus-names-4.3.4
                                                 rust-zcheapstr-1.1.0
                                                 rust-zvariant-5.15.0
                                                 rust-zvariant-derive-5.15.0
                                                 rust-zvariant-utils-4.2.0)))

(package
  (name "xdg-desktop-portal-logind")
  (version "0.1.0")
  (source (local-file %source-directory
                      "xdg-desktop-portal-logind-checkout"
                       #:recursive? #t
                       #:select?
                       (lambda (file stat)
                         (and (or (not %git-file?)
                                  (%git-file? file stat))
                              (not (string-suffix? ".scm" file))))))
  (build-system cargo-build-system)
  (arguments
   (list #:install-source? #f
         #:cargo-test-flags ''("--workspace" "--all-targets")
         #:phases
         #~(modify-phases %standard-phases
             (replace 'install
               (lambda* (#:key outputs #:allow-other-keys)
                 (invoke "cargo" "run" "--offline" "--package" "xtask" "--"
                         "install"
                         "--binary" "target/release/xdg-desktop-portal-logind"
                         "--prefix" (assoc-ref outputs "out")))))))
  (native-inputs (lookup-cargo-inputs 'xdg-desktop-portal-logind))
  (home-page #f)
  (synopsis "Portal backend for logind sleep and idle inhibition")
  (description
   "This package provides an XDG desktop portal Inhibit backend that translates
suspend and idle requests into systemd-logind or elogind block inhibitors.  Each
request owns an inhibitor file descriptor until it is closed.  The backend is
D-Bus-activated and does not require a Wayland connection or systemd as the init
system.")
  (license license:expat))
