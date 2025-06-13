import { invoke } from "@tauri-apps/api/core";
import { readFile, BaseDirectory } from "@tauri-apps/plugin-fs";
import { toast } from "svelte-sonner";

export function useMeetingData(meetingId: string) {
	// Create reactive state object
	const state = {
		transcriptContent: "",
		transcriptJsonContent: null as string | null,
		summaryContent: null as string | null,
		audio: null as Uint8Array | null,
		meetingMetadata: {} as { name?: string },
	};

	function getAudioURL() {
		if (!state.audio) return "";
		const blob = new Blob([state.audio], { type: "audio/ogg" });
		return URL.createObjectURL(blob);
	}

	async function getTranscript() {
		try {
			state.transcriptContent = await invoke("get_meeting_transcript", { meetingId });
			return state.transcriptContent;
		} catch (error) {
			console.error("Error fetching transcript:", error);
			throw error;
		}
	}

	async function getTranscriptJson() {
		try {
			state.transcriptJsonContent = await invoke("get_meeting_transcript_json", { meetingId });
			return state.transcriptJsonContent;
		} catch (error) {
			console.error("Error fetching transcript JSON:", error);
			state.transcriptJsonContent = null;
			throw error;
		}
	}

	async function getSummary() {
		try {
			state.summaryContent = await invoke("get_meeting_summary", { meetingId });
			return state.summaryContent;
		} catch (error) {
			console.error("Error fetching summary:", error);
			state.summaryContent = null;
			throw error;
		}
	}

	async function getAudio() {
		try {
			const audioData = await readFile(`uploads/${meetingId}/${meetingId}.ogg`, {
				baseDir: BaseDirectory.AppLocalData,
			});
			state.audio = audioData;
			return audioData;
		} catch (error) {
			console.error("Error fetching audio:", error);
			state.audio = null;
			throw error;
		}
	}

	async function getMeetingMetadata() {
		try {
			state.meetingMetadata = await invoke("get_meeting_metadata", { meetingId });
			return state.meetingMetadata;
		} catch (error) {
			console.error("Error fetching meeting metadata:", error);
			state.meetingMetadata = {};
			throw error;
		}
	}

	async function regenerateSummary() {
		try {
			state.summaryContent = await invoke("generate_summary", { meetingId });
			await getMeetingMetadata();
			return state.summaryContent;
		} catch (error) {
			console.error("Error regenerating summary:", error);
			toast.error("Error regenerating summary: " + error);
			throw error;
		}
	}

	async function transcribe() {
		try {
			await invoke("transcribe_with_chunking", { meetingId });
			await getTranscript();
			await getTranscriptJson();
			await getSummary();
		} catch (error) {
			console.error("Error starting transcription:", error);
			toast.error("Error starting transcription: " + error);
			throw error;
		}
	}

	return {
		// State - direct access to reactive state
		get transcriptContent() { return state.transcriptContent; },
		set transcriptContent(value: string) { state.transcriptContent = value; },
		get transcriptJsonContent() { return state.transcriptJsonContent; },
		get summaryContent() { return state.summaryContent; },
		get audio() { return state.audio; },
		get audioURL() { return getAudioURL(); },
		get meetingMetadata() { return state.meetingMetadata; },

		// Actions
		getTranscript,
		getTranscriptJson,
		getSummary,
		getAudio,
		getMeetingMetadata,
		regenerateSummary,
		transcribe,
	};
}
