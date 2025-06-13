import { listen, once } from "@tauri-apps/api/event";
import { toast } from "svelte-sonner";
import type { useProgressTracking } from "./useProgressTracking.svelte";

export function useEventListeners(
	meetingId: string,
	progressTracking: ReturnType<typeof useProgressTracking>,
	callbacks: {
		onTranscriptionStarted?: () => void;
		onTranscriptionFinished?: () => void;
		onSummarizationStarted?: (meetingId: string) => void;
	}
) {
	let listeners: (() => void)[] = [];

	async function setupListeners() {
		// WhisperX progress events
		const whisperxStartListener = await listen<number>("whisperx-start", (event) => {
			console.log("WhisperX started with", event.payload, "chunks");
			progressTracking.startTranscription(event.payload);
		});

		const whisperxProgressListener = await listen<number>("whisperx-progress", (event) => {
			console.log("WhisperX progress:", event.payload + 1, "of", progressTracking.transcriptionProgress.totalChunks);
			progressTracking.updateTranscriptionProgress(event.payload);
		});

		// FFmpeg progress events
		const ffmpegStartListener = await listen<number>("ffmpeg-start", (event) => {
			console.log("FFmpeg started with", event.payload, "chunks");
			progressTracking.startAudioSplitting(event.payload);
		});

		const ffmpegProgressListener = await listen<number>("ffmpeg-progress", (event) => {
			console.log("FFmpeg progress:", event.payload + 1, "of", progressTracking.audioSplittingProgress.totalChunks);
			progressTracking.updateAudioSplittingProgress(event.payload);
		});

		// Summarization progress events
		const summarizationChunkStartListener = await listen<number>("summarization-chunk-start", (event) => {
			console.log("Summarization started with", event.payload, "chunks");
			progressTracking.startSummarization(event.payload);
		});

		const summarizationChunkProgressListener = await listen<number>("summarization-chunk-progress", (event) => {
			console.log("Summarization progress:", event.payload + 1, "of", progressTracking.summarizationProgress.totalChunks);
			progressTracking.updateSummarizationProgress(event.payload);
		});

		// Meeting-specific events
		const summarizationListener = await once<string>("summarization-started", (event) => {
			console.log(event);
			console.log("Summarization started for meeting ID:", event.payload);
			toast.info("Summarization started: " + event.payload);
			callbacks.onSummarizationStarted?.(event.payload);
		});

		const transcriptionListener = await listen<string>(meetingId, (event) => {
			if (event.payload === "transcription-started") {
				toast.info("Transcription started for meeting ID: " + meetingId);
				callbacks.onTranscriptionStarted?.();
			} else if (event.payload === "transcription-finished") {
				toast.success("Transcription finished for meeting ID: " + meetingId);
				callbacks.onTranscriptionFinished?.();
			}
		});

		// Store listeners for cleanup
		listeners = [
			whisperxStartListener,
			whisperxProgressListener,
			ffmpegStartListener,
			ffmpegProgressListener,
			summarizationChunkStartListener,
			summarizationChunkProgressListener,
			summarizationListener,
			transcriptionListener,
		];
	}

	function cleanup() {
		listeners.forEach((listener) => listener());
		listeners = [];
	}

	return {
		setupListeners,
		cleanup,
	};
}
