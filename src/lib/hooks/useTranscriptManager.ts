import { invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";
import type { useMeetingData } from "./useMeetingData.svelte";

export function useTranscriptManager(meetingId: string, meetingData: ReturnType<typeof useMeetingData>) {
	async function checkTranscriptionStatus() {
		try {
			const status = await invoke("is_transcribing", { meetingId });
			console.log("Transcription status for meeting ID", meetingId, ":", status);
			return status;
		} catch (error) {
			toast.error("Error checking transcription status: " + error);
			console.error("Error checking transcription status:", error);
			throw error;
		}
	}

	async function handleTranscriptLoad() {
		try {
			await meetingData.getTranscript();
			return true;
		} catch (error) {
			console.error("Error fetching transcript:", error);

			const isTranscribing = await checkTranscriptionStatus();
			if (isTranscribing && isTranscribing !== meetingId) {
				toast.info("Another Transcription is still in progress. Please wait.");
				return false;
			} else if (isTranscribing === meetingId) {
				toast.info("Transcription is still in progress. Please wait.");
				return false;
			}

			// Auto-start transcription if transcript doesn't exist
			try {
				await invoke("transcribe_with_chunking", { meetingId });
				console.log("Transcription finished successfully");
				await meetingData.getTranscript();
				await meetingData.getTranscriptJson();
				await meetingData.getSummary();
				return true;
			} catch (transcribeError) {
				console.error("Error transcribing audio:", transcribeError);
				toast.error("Error starting transcription: " + transcribeError);
				return false;
			}
		}
	}

	async function reloadTranscript() {
		try {
			await meetingData.getTranscript();
			await meetingData.getTranscriptJson();
			toast.success("Transcript reloaded successfully!");
			return true;
		} catch (error) {
			console.error("Error reloading transcript:", error);
			toast.error("Error reloading transcript: " + error);
			return false;
		}
	}

	return {
		checkTranscriptionStatus,
		handleTranscriptLoad,
		reloadTranscript,
	};
}
