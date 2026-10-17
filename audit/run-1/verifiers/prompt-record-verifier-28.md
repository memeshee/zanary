# Phase-5 record verifier — verifier-28 on `video/zanary-demo.mp4:publish-video-unverifiable-media-content`

You are a fresh `research` verifier with independent eyes on ONE finished record of the zanary run-1
audit. You did not hunt this candidate and you did not perform its Phase-3 validation. You check the
record itself (not any hunter write-up) and stay inside source/local boundaries.

Run directory: `/home/kiter/security-audit-skill/zanary/run-1` (outside the target). Target: `/home/kiter/hackathon/zanary` (repo `memeshee/zanary`, commit
`7058406`, worktree clean) — READ-ONLY: never modify a target file.

## The record you must check (`needs_validation`)

```json
{
  "verdict": "needs_validation",
  "fingerprint": "video/zanary-demo.mp4:publish-video-unverifiable-media-content",
  "title": "Publish-video step serves a media asset whose disclosed content no repository artifact constrains or verifies",
  "description": "The publish job copies video/*.mp4 verbatim into the repo's world-readable Pages site (.github/workflows/privacy-gate.yml:172, 209-215) and the player page asserts \"What is on screen comes from a real run\" (:198). At the reviewed commit the glob matches exactly one file: video/zanary-demo.mp4 (3717435 B; sha256 38a6b73fb25bab4e3888988639658e603336a8784e5503af7783179cb159ec75; git blob df469b5a2077ae94fd86b0ddb44535bd6c4e9f38) - a 112 s rendered cut of a local rehearsal take whose source log and whole recording/rendering pipeline sit outside the repository by stated design (docs/VIDEO-SCRIPT.md:8-9; privacy-gate.yml:169-170). Re-verified in the approved sandbox: the container carries only an avc1 video track and an mp4a audio track, no text/subtitle/chapter track, no bytes after the last top-level box, and no metadata beyond encoder tags - so the remaining disclosure surface is exactly the decoded pixels and the narration audio, and the approved sandbox has no decoder for them. Source eliminates the sharpest single example (the only secret the CLI prints by design is the dev seed phrase, itself the canonical BIP-39 test vector already committed in-tree and never invoked by the demo scripts) but constrains nothing else: whether the 112 s carries only the intended-public run data enumerated by the cut sheet (docs/VIDEO-SCRIPT.md:34-99) or also values a manual screen recording can capture (operator paths/usernames, anything else the take showed) is decisive for the world-readable-Pages disclosure boundary and is not establishable from source or the approved sandbox.",
  "claimed_root_cause": "The publisher ships a committed binary media asset to a world-readable site on the strength of a manual, out-of-tree review: no in-tree control (hash pin on the published bytes, content scan, per-take review, cut-sheet diff) constrains what the frames and audio of video/*.mp4 show, and the producer pipeline and its source take are deliberately outside the repository - so the artifact's actual disclosure content is unverified by construction, not by policy.",
  "trace": [
    {
      "kind": "entrypoint",
      "file": "video/zanary-demo.mp4",
      "line": 1,
      "scope": "the committed media asset (ISO-BMFF ftyp isom/iso2/avc1/mp41: one avc1 video track + one mp4a audio track, 112.13 s, 3717435 B, sha256 38a6b73f..., git blob df469b5a...)",
      "description": "The disclosure surface: a rendered cut whose on-screen content is claimed in docs/VIDEO-SCRIPT.md but not derivable or checkable from any repository artifact; verifier-9 re-checked it byte-identical to the reviewed revision's committed blob."
    },
    {
      "kind": "propagation",
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 9,
      "scope": "cut sheet - origin of every on-screen block",
      "description": "\"every line below is verbatim from ~/hackathon/zanary-rehearsal.log, a real take that exited 0\" (:8-9) - the content's source is a log outside the reviewed revision, and no TTS/frame-render/ffmpeg step exists anywhere in the tree."
    },
    {
      "kind": "propagation",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 170,
      "scope": "publish job, comment above the demo step",
      "description": "The workflow itself records that the recording pipeline stays out of the repository, so no CI step can re-derive or check the media content."
    },
    {
      "kind": "propagation",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 172,
      "scope": "publish job, step 'publish the two-minute demo next to the claims'",
      "description": "cp video/*.mp4 public/video/ - unconditional verbatim copy with no check; the publish job runs on main pushes and the weekly scheduled re-probe (:16-23, :113) and at this commit the glob matches exactly one file."
    },
    {
      "kind": "sink",
      "file": ".github/workflows/privacy-gate.yml",
      "line": 215,
      "scope": "uses: actions/upload-pages-artifact (:209) + actions/deploy-pages (:215)",
      "description": "public/ is deployed to memeshee.github.io/zanary/video/ - world-readable and submission-critical (docs/SUBMISSION.md:99,111); no content review stands between the committed bytes and the world."
    }
  ],
  "evidence": [
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 172,
      "description": "Verbatim copy of every video/*.mp4 into the published site: no hash pin (grep of the tree for the asset's sha256: 0 hits), no scan, no transform, no step reads the bytes; the \"|| echo no video in the tree - skipping\" fallback also lets a missing or renamed file deploy a player page that 404s without failing the job."
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 170,
      "description": "\"The recording pipeline itself stays out of the repository\" (:169-170) - the producer of the published bytes is not reviewable in-tree."
    },
    {
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 8,
      "description": "\"Nothing is staged: every line below is verbatim from ~/hackathon/zanary-rehearsal.log, a real take that exited 0\" (:8-9) - the asset's central content claim rests on an out-of-tree log; source contains no render/TTS/ffmpeg pipeline."
    },
    {
      "file": "docs/VIDEO-SCRIPT.md",
      "line": 34,
      "description": "The enumerated intended on-screen blocks (typed command; run output; payment lines with txids/heights; property table; claim block, :34-46) plus the payer ledger and kill-switch block (:55-68), the control 'caught' block (:76-82) and the index/Actions shots (:90) - the intended-public content list a frame/audio review would diff against."
    },
    {
      "file": ".github/workflows/privacy-gate.yml",
      "line": 198,
      "description": "The published page asserts \"What is on screen comes from a real run ...\" (:198-200) - a content claim the repository cannot verify; the only in-tree references to the media are the player (src hardcoded at :196), this publish step, README.md:15 and docs/SUBMISSION.md:99/111."
    },
    {
      "file": "crates/properties/src/properties/report_leak.rs",
      "line": 82,
      "description": "report-leak - the project's excess-disclosure rule (discloses list, key-material tokens :26-39, unknown addresses and emails :125-138) - inspects run.receipts only (skip when empty :82-89; loop :107) and never reaches the media asset; no other in-tree rule or CI check reviews the video's content."
    },
    {
      "file": "video/zanary-demo.mp4",
      "line": 1,
      "description": "verifier-9 in-sandbox re-check of the committed bytes (rust:1-slim, --network=none; git-blob sha1 df469b5a... = the committed blob): sha256 38a6b73f..., 3717435 B; independent box walk: ftyp+moov+free+mdat only, two tracks (hdlr vide/stsd avc1; hdlr soun/stsd mp4a), 112.13 s, no text/subtitle/chapter box (raw counts 0), top-level end == file length (0 trailing bytes), creation/mod 0, udta/meta only (c)too=Lavf63.1.101; raw-byte token scan 0 hits; the undecoded pixels/audio remain the only unexamined region (verifier-9 check-video-container.txt)."
    },
    {
      "file": "crates/probe/src/wallet.rs",
      "line": 38,
      "description": "The CLI prints the dev seed phrase by design (:38) - but the value is the canonical BIP-39 English test vector, already committed (crates/probe/src/core.rs:63; docs/WALLET-NOTES.md), and neither demo script invokes `wallet` (scripts/demo-run.sh:107-125, scripts/bounded-payer-demo.sh:59-64) - source eliminates this example; what remains pixel-only is any other value the take captured."
    },
    {
      "file": "docs/SUBMISSION.md",
      "line": 111,
      "description": "The ship checklist marks the video built and served (\"112 s, frames verbatim from a real run\"), and :99 pins the direct file URL .../video/zanary-demo.mp4; the publication is deliberate and submission-critical - which is exactly why its unverified content is the question."
    }
  ],
  "blockers": [
    "Decode is impossible inside the approved sandbox: rust:1-slim carries no ffmpeg/ffprobe/avconv/gstreamer/vlc/mpv/python3/ImageMagick/exiftool/mediainfo/tesseract/sox and no libav*/gstreamer libs; --network=none forbids fetching one; image pulls are out of policy; and the read-only offline registry (418 cached crates) contains no media-capable crate (the only grep hits were rustls-webpki/webpki-roots, TLS crates). Running host tooling on the fixture is barred too - it would process target-controlled bytes outside the OS-enforced sandbox (verifier-9 check-video-container.txt).",
    "The producer is out of tree: the source take (~/hackathon/zanary-rehearsal.log) and the TTS/frame-render/ffmpeg pipeline are outside the reviewed revision by stated design (docs/VIDEO-SCRIPT.md:8-9; privacy-gate.yml:169-170) and no render, TTS or ffmpeg step exists anywhere in the tree, so the frames cannot be re-derived from the repository or diffed against a committed take.",
    "The take cannot be re-recorded to compare: the regtest rig's containers and host ports (18232/18233/9067/9068) are out of bounds for this audit, and the recording procedure depends on that rig."
  ],
  "validation_plan": {
    "local": "Owner-side, offline, one bounded pass over a copy of the committed regular file (sha256 38a6b73f..., 3717435 B, git blob df469b5a...; never modify the reviewed tree): decode frames (e.g. ffmpeg -i copy.mp4 -vf fps=2 frames/%04d.png -> ~225 frames for 112 s) and OCR every frame; decode the audio to WAV and transcribe it; diff every extracted token and audio line against the cut sheet's intended-public block list (docs/VIDEO-SCRIPT.md:34-99) and the take's own log; fail on any report-leak token class (ivk/ovk/ufvk/fvk/viewing key/spending key/seed phrase/mnemonic), any absolute host path or username, any address or long hex string absent from the run's own records, or audio beyond the narration script; inspect the first and last frames for desktop content outside the terminal.",
    "deployment": "Owner-observed on the published site (observation only, not a probe of a live target): confirm the bytes served at memeshee.github.io/zanary/video/zanary-demo.mp4 hash to the reviewed revision's sha256 (38a6b73f...) and the player page loads exactly that one file; if any frame or audio value outside the claim is found, replace the asset and re-deploy - the published copy (Pages, git history, crawler mirrors) cannot be recalled piecewise, only rewritten."
  }
}
```

## Linked ledger unit

- coverage_id: `video%2Fzanary-demo.mp4%23published-asset::.github%2Fworkflows%2Fprivacy-gate.yml%23publish-video::published-evidence::ATTACK-CLASSES.md%23Feature%20abuse%20and%20data%20leakage`
- surface: `video/zanary-demo.mp4#published-asset` · subsystem `published-evidence`
- boundary: `.github/workflows/privacy-gate.yml#publish-video`
- assignment owner: `hunter-video`

The unit's checks (each `artifact` path is relative to the run directory; read the promoted files
under `/home/kiter/security-audit-skill/zanary/run-1/agents/<check-owner>/artifacts/`):

```json
[
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      ".github/workflows/privacy-gate.yml"
    ],
    "invariant": "The publish-video step reproduces exactly the reviewed revision's committed media bytes into the Pages output path: a verbatim local copy, gated to main after the gate jobs, with no network fetch, transform, or content-dependent branch a contributor could steer.",
    "method": "source",
    "result": "Lines 166-172 copy video/*.mp4 to public/video/ verbatim (no hash pin, no scan, no transform); the publish job runs only when github.ref == 'refs/heads/main' (line 113) and after needs: gate; lines 209-215 upload public/ and deploy it; the player page hardcodes the served file name zanary-demo.mp4 (line 196). Nothing else decides what is served, and at the reviewed commit the glob matches exactly one file (video/zanary-demo.mp4, 3717435 B).",
    "artifact": null
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "docs/VIDEO-SCRIPT.md",
      "README.md",
      "docs/SUBMISSION.md"
    ],
    "invariant": "What the published asset is claimed to show, and where its on-screen text comes from, are stated in-tree well enough to name the intended-public content the pixels would have to match.",
    "method": "source",
    "result": "The cut sheet claims a 112 s cut of one real exit-0 take, every on-screen block verbatim from an out-of-tree rehearsal log (lines 8-9), and enumerates the blocks (typed command, payment lines with txids/heights, property table, claim block, payer ledger + kill-switch block, control 'caught' block, index and Actions shots). README lines 15-18 and SUBMISSION.md lines 99/111 surface the same claims to judges. Source states the claim but cannot exhibit the pixels; the take log and the render pipeline (TTS beats, frame renderer, ffmpeg) are out of tree by stated design (privacy-gate.yml:169-170).",
    "artifact": null
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "video/zanary-demo.mp4"
    ],
    "invariant": "The published container carries no stowaway payload: no text/subtitle/chapter track, no bytes after the last top-level box, no author/path/creation metadata, and no plaintext key-material or address-shaped strings in any uncompressed region.",
    "method": "local",
    "result": "Sandboxed parse (rust:1-slim, --network=none, disposable copy) of the committed bytes (sha256 38a6b73fb25bab4e3888988639658e603336a8784e5503af7783179cb159ec75; 3717435 B): one avc1 video track (112.13 s, Lavc63.1.101 libx264) plus one mp4a audio track (112.15 s); no text/subtitle/chapter track; the file ends exactly at the end of the last top-level box (0 trailing bytes, no appended polyglot payload); creation/modification times 0; udta/meta carries only \u00a9too=Lavf63.1.101. Raw-byte token scan: 0 hits for abandon / seed phrase / mnemonic / ufvk / uview / rpcpassword / zanary-rehearsal / kiter / uregtest / allowlist. Decoder probe: no ffmpeg/ffprobe/python3 in the sandbox and no media crates in the read-only offline registry, so decoded frames/audio - the only remaining content surface - are not inspectable.",
    "artifact": "agents/hunter-video/artifacts/check-video-container.txt"
  },
  {
    "agent_id": "hunter-video",
    "reviewed_paths": [
      "crates/properties/src/properties/report_leak.rs",
      "docs/PROPERTY-IMPL-NOTES.md",
      "docs/THREAT-MODEL.md",
      "docs/CLAIMS.md"
    ],
    "invariant": "One of the project's own disclosure rules for published artifacts reaches the media asset (no key-material tokens, no disclosure beyond a claim), or their absence is explicit and acknowledged.",
    "method": "source",
    "result": "report-leak - the excess-disclosure rule (discloses list, key-material tokens, addresses not in the run's records, emails) - inspects flow-exported receipts only and skips runs without receipts (report_leak.rs:81-89); the threat model's disclosure inventory (THREAT-MODEL.md:3, 43-46) and CLAIMS.md's publishing rules (36-57) cover claim artifacts, not the video or the Pages video publish. No in-tree rule or CI check reviews the media content.",
    "artifact": null
  }
]
```

## What to check for this record

1. The source path is real and supports only the `claimed_root_cause` stated.
2. Every listed blocker is decisive and not already answerable locally.
3. The candidate names a boundary and a possible concrete result rather than a generic concern.
4. At least one `validation_plan` field is present and exact: `local` uses a bounded fixture; `deployment` asks the owner to observe a configuration, identity, route, policy or runtime fact. Do not invent a plan for an inapplicable context, and never send audit traffic to a deployment.
5. The fingerprint matches prior/current records for the same root cause.

## Boundaries

Target-controlled code may run only inside the approved sandbox. The approved disposable copy is
`/home/kiter/.hermes/cache/scratch/zanary-sandbox-w2` (`git archive HEAD`, `rust-toolchain.toml` removed so the image's own toolchain is used).
Rebuild it if needed: `rm -rf /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && mkdir -p /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && cd /home/kiter/hackathon/zanary && git archive HEAD | tar -x -C /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2 && rm -f /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2/rust-toolchain.toml`

```bash
docker run --rm --network=none --cpus=6 --memory=3500m --memory-swap=5g --pids-limit=1024 \
  -v /home/kiter/.hermes/cache/scratch/zanary-sandbox-w2:/t -v /home/kiter/.cargo/registry:/usr/local/cargo/registry:ro -w /t rust:1-slim \
  sh -c 'CARGO_BUILD_JOBS=6 <command>'
```

No external network, no image pulls, no `sudo`, no host paths besides that copy and the read-only
registry; writes only inside that copy and your own scratch. Out of bounds: the regtest rig's
containers and host ports (18232/18233/9067/9068), any network service, any public URL, and anything
that mutates state outside your scratch. If a decisive fact lives behind one of those, say so in your
`replace` reason rather than reaching past the boundary.

Your write paths: scratch `/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-28/scratch/` (scratch-relative files only: `evidence.txt`,
`check-*.txt`, `check-*.json`; 64 KiB per file, 256 KiB total). The parent promotes into
`/home/kiter/security-audit-skill/zanary/run-1/agents/verifier-28/artifacts/`. You never write retained artifacts, another agent's directory, or a
shared run file.

## Schema branches (verbatim from `report-schema.json`)

`confirmed`:

```json
{
  "type": "object",
  "description": "A source-grounded vulnerability that was independently demonstrated.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "confirmed"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$",
      "description": "A stable source-derived identifier that does not change between validation states."
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "intended_behavior": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "conditions": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "authentication_level",
              "authorization_role",
              "user_interaction",
              "system_configuration",
              "network_routing",
              "environmental_dependency",
              "data_state",
              "timing_dependency",
              "third_party_dependency"
            ]
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "execution": {
      "type": "object",
      "description": "Target-neutral reproduction in the target's native interface.",
      "properties": {
        "attacker_perspective": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "payloads": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {
            "type": "string"
          }
        },
        "instructions": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "observed_result": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "attacker_perspective",
        "payloads",
        "instructions",
        "observed_result"
      ],
      "additionalProperties": false
    },
    "remediation": {
      "type": "object",
      "properties": {
        "strategy": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "code_changes": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "file_name": {
                "type": "string",
                "minLength": 1
              },
              "fixed_code": {
                "type": "string"
              }
            },
            "required": [
              "file_name",
              "fixed_code"
            ],
            "additionalProperties": false
          }
        }
      },
      "required": [
        "strategy"
      ],
      "additionalProperties": false
    },
    "severity": {
      "type": "object",
      "properties": {
        "likelihood": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "impact": {
          "type": "object",
          "properties": {
            "score": {
              "type": "string",
              "enum": [
                "informational",
                "low",
                "medium",
                "high",
                "critical"
              ]
            },
            "reason": {
              "type": "string",
              "minLength": 1,
              "visibleContent": true
            }
          },
          "required": [
            "score",
            "reason"
          ],
          "additionalProperties": false
        },
        "overall_severity": {
          "type": "string",
          "enum": [
            "informational",
            "low",
            "medium",
            "high",
            "critical"
          ]
        }
      },
      "required": [
        "likelihood",
        "impact",
        "overall_severity"
      ],
      "additionalProperties": false
    },
    "confidence": {
      "type": "object",
      "properties": {
        "score": {
          "type": "string",
          "enum": [
            "low",
            "medium",
            "high"
          ]
        },
        "reason": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "required": [
        "score",
        "reason"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "root_cause",
    "intended_behavior",
    "trace",
    "evidence",
    "conditions",
    "execution",
    "remediation",
    "severity",
    "confidence"
  ],
  "additionalProperties": false
}
```

`needs_validation`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate whose decisive validation is blocked.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "needs_validation"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "blockers": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "string",
        "minLength": 1,
        "visibleContent": true
      }
    },
    "validation_plan": {
      "type": "object",
      "properties": {
        "local": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        },
        "deployment": {
          "type": "string",
          "minLength": 1,
          "visibleContent": true
        }
      },
      "additionalProperties": false
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "blockers",
    "validation_plan"
  ],
  "additionalProperties": false
}
```

`rejected`:

```json
{
  "type": "object",
  "description": "A source-grounded candidate refuted during validation.",
  "properties": {
    "verdict": {
      "type": "string",
      "const": "rejected"
    },
    "fingerprint": {
      "type": "string",
      "minLength": 1,
      "pattern": "^[A-Za-z0-9][A-Za-z0-9._:/@+-]*$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "description": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "claimed_root_cause": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    },
    "trace": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "enum": [
              "entrypoint",
              "propagation",
              "sink"
            ]
          },
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "scope": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "kind",
          "file",
          "line",
          "scope",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "evidence": {
      "type": "array",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "minLength": 1
          },
          "line": {
            "type": "integer",
            "minimum": 1
          },
          "description": {
            "type": "string",
            "minLength": 1,
            "visibleContent": true
          }
        },
        "required": [
          "file",
          "line",
          "description"
        ],
        "additionalProperties": false
      }
    },
    "reason": {
      "type": "string",
      "minLength": 1,
      "visibleContent": true
    }
  },
  "required": [
    "verdict",
    "fingerprint",
    "title",
    "description",
    "claimed_root_cause",
    "trace",
    "evidence",
    "reason"
  ],
  "additionalProperties": false
}
```

## Your result

Return exactly one JSON object, no prose and no code fences:

- `{"decision": "verified", "fingerprint": "video/zanary-demo.mp4:publish-video-unverifiable-media-content"}` when the record holds as written (non-material
  wording or line corrections may be reported in a short `notes` field, but the parent applies only
  changes that do not alter meaning or evidence);
- `{"decision": "replace", "reason": "why the record is wrong", "record": { ... }}` when it does
  not — the replacement must match the `confirmed`, `needs_validation` or `rejected` branch above
  exactly and keep the same fingerprint unless you identified a genuinely different root cause (say so
  in `reason`).

Do not strengthen a verdict: a replacement that promotes the record to a stronger verdict, or that
materially changes the root cause, trace, execution input, observed result, impact or severity, will
itself be sent for another independent verification by the parent.
