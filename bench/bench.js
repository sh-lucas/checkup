/**
 * k6 benchmark — mixed load: heavy joins + light reads + writes, all simultaneous.
 * Usage: BASE_URL=http://localhost:3000 k6 run bench/bench.js
 */
import http from "k6/http";
import { check } from "k6";

const BASE = __ENV.BASE_URL || "http://localhost:3000";

export const options = {
  scenarios: {
    heavy_reads: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 8 },
        { duration: "4m",  target: 8 },
        { duration: "30s", target: 0 },
      ],
      exec: "heavy",
      tags: { scenario: "heavy" },
    },
    light_reads: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 8 },
        { duration: "4m",  target: 8 },
        { duration: "30s", target: 0 },
      ],
      exec: "light",
      tags: { scenario: "light" },
    },
    writes: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 4 },
        { duration: "4m",  target: 4 },
        { duration: "30s", target: 0 },
      ],
      exec: "write",
      tags: { scenario: "write" },
    },
  },
  thresholds: {
    "http_req_duration{scenario:heavy}": ["p(95)<2000"],
    "http_req_duration{scenario:light}": ["p(95)<500"],
    "http_req_duration{scenario:write}": ["p(95)<500"],
    http_req_failed: ["rate<0.01"],
  },
};

export function heavy() {
  const res = http.get(`${BASE}/bench/heavy`);
  check(res, { "200": (r) => r.status === 200 });
}

export function light() {
  const res = http.get(`${BASE}/bench/light`);
  check(res, { "200": (r) => r.status === 200 });
}

export function write() {
  const res = http.post(`${BASE}/bench/write`, null);
  check(res, { "200": (r) => r.status === 200 });
}
