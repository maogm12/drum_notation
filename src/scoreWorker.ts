/// <reference lib="webworker" />

import { buildMusicXml, buildNormalizedScore, type ParseMode } from "./dsl";

type ParseRequest = {
  type: "parse";
  id: number;
  dsl: string;
  hideVoice2Rests: boolean;
  parseMode: ParseMode;
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
  score: ReturnType<typeof buildNormalizedScore>;
};

type XmlResponse = {
  type: "xml";
  id: number;
  xml: string;
};

let lastScore: ReturnType<typeof buildNormalizedScore> | null = null;

self.onmessage = (event: MessageEvent<ScoreWorkerRequest>) => {
  const msg = event.data;

  if (msg.type === "parse") {
    const { id, dsl, parseMode } = msg;
    const score = buildNormalizedScore(dsl, parseMode);
    lastScore = score;
    const response: ParseResponse = { type: "parse", id, score };
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
};

export {};
