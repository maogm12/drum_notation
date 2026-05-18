/// <reference lib="webworker" />

import { buildMusicXml, buildNormalizedScore } from "./dsl";
import { initParserWasmBrowser } from "./wasm/parser_wasm_browser";

const wasmInit = initParserWasmBrowser();

type ParseRequest = {
  type: "parse";
  id: number;
  sourceRevision: number;
  dsl: string;
  hideVoice2Rests: boolean;
};

type GenerateXmlRequest = {
  type: "generateXml";
  id: number;
  hideVoice2Rests: boolean;
};

type ScoreWorkerRequest = ParseRequest | GenerateXmlRequest;

type ParseResponse = {
  type: "parse";
  id: number;
  source: string;
  sourceRevision: number;
  score: ReturnType<typeof buildNormalizedScore>;
};

type XmlResponse = {
  type: "xml";
  id: number;
  xml: string;
};

let lastScore: ReturnType<typeof buildNormalizedScore> | null = null;

function handleMessage(msg: ScoreWorkerRequest) {
  if (msg.type === "parse") {
    const { id, dsl, sourceRevision } = msg;
    const score = buildNormalizedScore(dsl);
    lastScore = score;
    const response: ParseResponse = {
      type: "parse",
      id,
      source: dsl,
      sourceRevision,
      score,
    };
    self.postMessage(response);
  } else if (msg.type === "generateXml") {
    const { id, hideVoice2Rests } = msg;
    if (!lastScore) {
      const response: XmlResponse = { type: "xml", id, xml: "" };
      self.postMessage(response);
      return;
    }
    const xml = buildMusicXml(lastScore, hideVoice2Rests);
    const response: XmlResponse = { type: "xml", id, xml };
    self.postMessage(response);
  }
}

self.onmessage = async (event: MessageEvent<ScoreWorkerRequest>) => {
  const msg = event.data;
  if (msg.type === "parse") {
    await wasmInit;
  }
  handleMessage(msg);
};

export {};
