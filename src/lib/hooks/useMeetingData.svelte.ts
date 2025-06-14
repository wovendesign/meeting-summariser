import { invoke } from "@tauri-apps/api/core";
import { readFile, BaseDirectory } from "@tauri-apps/plugin-fs";
import { toast } from "svelte-sonner";

interface ChunkSummary {
  chunk_number: number;
  content: string;
  markdown_content: string;
}

export function useMeetingData(meetingId: string) {
  let transcriptContent = $state("");
  let transcriptJsonContent: string | null = $state(null);
  let summaryContent: string | null = $state("");
  let chunkSummaries: ChunkSummary[] = $state([]);
  let audio: Uint8Array | null = $state(null);
  let meetingMetadata: { name?: string } = $state({});

  const audioURL = $derived.by(() => {
    if (!audio) return "";
    const blob = new Blob([audio], { type: "audio/ogg" });
    return URL.createObjectURL(blob);
  });

  async function getTranscript() {
    try {
      transcriptContent = await invoke("get_meeting_transcript", { meetingId });
      return transcriptContent;
    } catch (error) {
      console.error("Error fetching transcript:", error);
      throw error;
    }
  }

  async function getTranscriptJson() {
    try {
      transcriptJsonContent = await invoke("get_meeting_transcript_json", { meetingId });
      return transcriptJsonContent;
    } catch (error) {
      console.error("Error fetching transcript JSON:", error);
      transcriptJsonContent = null;
      throw error;
    }
  }

  async function getSummary() {
    try {
      summaryContent = await invoke("get_meeting_summary", { meetingId });
      return summaryContent;
    } catch (error) {
      console.error("Error fetching summary:", error);
      summaryContent = null;
      throw error;
    }
  }

  async function getChunkSummaries() {
    try {
      chunkSummaries = await invoke("get_chunk_summaries", { meetingId });
      return chunkSummaries;
    } catch (error) {
      console.error("Error fetching chunk summaries:", error);
      chunkSummaries = [];
      throw error;
    }
  }

  async function getAudio() {
    try {
      const audioData = await readFile(`uploads/${meetingId}/${meetingId}.ogg`, {
        baseDir: BaseDirectory.AppLocalData,
      });
      audio = audioData;
      return audioData;
    } catch (error) {
      console.error("Error fetching audio:", error);
      audio = null;
      throw error;
    }
  }

  async function getMeetingMetadata() {
    try {
      meetingMetadata = await invoke("get_meeting_metadata", { meetingId });
      return meetingMetadata;
    } catch (error) {
      console.error("Error fetching meeting metadata:", error);
      meetingMetadata = {};
      throw error;
    }
  }

  async function regenerateSummary() {
    try {
      summaryContent = await invoke("generate_summary", { meetingId });
      await getMeetingMetadata();
      return summaryContent;
    } catch (error) {
      console.error("Error regenerating summary:", error);
      toast.error("Error regenerating summary: " + error);
      throw error;
    }
  }

  async function regenerateFinalSummary() {
    try {
      summaryContent = await invoke("regenerate_final_summary", { meetingId });
      await getMeetingMetadata();
      return summaryContent;
    } catch (error) {
      console.error("Error regenerating final summary:", error);
      toast.error("Error regenerating final summary: " + error);
      throw error;
    }
  }

  async function transcribe() {
    try {
      await invoke("transcribe_with_chunking", { meetingId });
      await getTranscript();
      await getTranscriptJson();
      await getSummary();
      await getChunkSummaries();
    } catch (error) {
      console.error("Error starting transcription:", error);
      toast.error("Error starting transcription: " + error);
      throw error;
    }
  }

  return {
    // State - direct access to reactive state
    get transcriptContent() { return transcriptContent; },
    set transcriptContent(value: string) { transcriptContent = value; },
    get transcriptJsonContent() { return transcriptJsonContent; },
    get summaryContent() { return summaryContent; },
    get chunkSummaries() { return chunkSummaries; },
    get audio() { return audio; },
    get audioURL() { return audioURL; },
    get meetingMetadata() { return meetingMetadata; },

    // Actions
    getTranscript,
    getTranscriptJson,
    getSummary,
    getChunkSummaries,
    getAudio,
    getMeetingMetadata,
    regenerateSummary,
    regenerateFinalSummary,
    transcribe,
  };
}
