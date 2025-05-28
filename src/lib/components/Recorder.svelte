<script lang="ts">
	import { onDestroy } from 'svelte';
	import SpeakerNaming from '../components/SpeakerNaming.svelte';
	import { writeFile, BaseDirectory, mkdir } from '@tauri-apps/plugin-fs';

	// State runes
	let mediaRecorder = $state<MediaRecorder | null>(null);
	let audioChunks = $state<Blob[]>([]);
	let recording = $state(false);
	let audioURL = $state('');
	let uploadStatus = $state('');
	let serverFilePath = $state('');
	let transcript = $state('');
	let isUploading = $state(false);
	let micLevel = $state(0);
	let transcribing = $state(false);
	let eventSource: EventSource | null = null;
	let summary = $state('');
	let summarizing = $state(false);

	// Audio analysis for mic level indicator
	let audioContext: AudioContext | null = null;
	let analyser: AnalyserNode | null = null;
	let animationFrame: number | null = null;

	// Derived state for button disabled states
	const startDisabled = $derived(recording || isUploading);
	const stopDisabled = $derived(!recording || isUploading);

	async function startRecording() {
		try {
			const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

			// Set up audio analysis for mic level
			audioContext = new AudioContext();

			// Resume audio context if it's suspended (required by some browsers)
			if (audioContext.state === 'suspended') {
				await audioContext.resume();
			}

			const source = audioContext.createMediaStreamSource(stream);
			analyser = audioContext.createAnalyser();
			analyser.fftSize = 2048; // Increase for better resolution
			analyser.smoothingTimeConstant = 0.3; // Smooth out the values
			source.connect(analyser);

			console.log('Audio context state:', audioContext.state);
			console.log('Analyser setup complete');

			mediaRecorder = new MediaRecorder(stream);

			mediaRecorder.ondataavailable = (event) => {
				audioChunks.push(event.data);
			};

			mediaRecorder.onstop = async () => {
				const audioBlob = new Blob(audioChunks, { type: 'audio/webm' });
				audioURL = URL.createObjectURL(audioBlob);
				await uploadToServer(audioBlob);
				audioChunks = [];

				// Clean up audio analysis
				if (animationFrame) {
					cancelAnimationFrame(animationFrame);
					animationFrame = null;
				}
				if (audioContext && audioContext.state !== 'closed') {
					audioContext.close();
				}
				audioContext = null;
				analyser = null;
				micLevel = 0;
			};

			mediaRecorder.start();
			recording = true;

			// Start analyzing audio levels AFTER setting recording to true
			updateMicLevel();

			// Reset status messages when starting a new recording
			uploadStatus = '';
			serverFilePath = '';
		} catch (error) {
			console.error('Error starting recording:', error);
			uploadStatus = `Error starting recording: ${error instanceof Error ? error.message : 'Unknown error'}`;
		}
	}

	function stopRecording() {
		if (mediaRecorder && recording) {
			mediaRecorder.stop();
			recording = false;
		}
	}

	function updateMicLevel() {
		if (!analyser || !recording) {
			console.log('No analyser or not recording');
			return;
		}

		const dataArray = new Uint8Array(analyser.fftSize);
		analyser.getByteTimeDomainData(dataArray);

		// Calculate RMS (Root Mean Square) for volume level
		let sum = 0;
		for (let i = 0; i < dataArray.length; i++) {
			const sample = (dataArray[i] - 128) / 128; // Convert to -1 to 1 range
			sum += sample * sample;
		}
		const rms = Math.sqrt(sum / dataArray.length);

		// Convert to percentage and apply some scaling for better visual feedback
		const newLevel = Math.min(100, rms * 300); // Increased scaling for better visibility
		micLevel = newLevel;

		// Debug logging (remove this later)
		if (Math.random() < 0.01) {
			// Log occasionally to avoid spam
			console.log('RMS:', rms, 'Level:', newLevel);
		}

		if (recording) {
			animationFrame = requestAnimationFrame(updateMicLevel);
		}
	}

	async function uploadToServer(blob: Blob) {
		try {
			isUploading = true;
			uploadStatus = 'Uploading recording to server...';

			const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
			const recordingName = `recording-${timestamp}`;

			const contents = new Uint8Array(await blob.arrayBuffer());

			// Make Dir
			await mkdir(`uploads/${recordingName}`, {
				baseDir: BaseDirectory.AppLocalData,
			});

			// Write File
			await writeFile(`uploads/${recordingName}/${recordingName}.webm`, contents, {
				baseDir: BaseDirectory.AppLocalData,
			});

		} catch (error) {
			uploadStatus = `Error uploading: ${error instanceof Error ? error.message : 'Unknown error'}`;
			console.error('Upload error:', error);
		} finally {
			isUploading = false;
		}
	}

	function startTranscription(filepath: string) {
		// Reset and open SSE connection
		transcript = '';
		transcribing = true;
		eventSource = new EventSource(`/api/transcribe?filepath=${encodeURIComponent(filepath)}`);
		eventSource.onmessage = (e) => {
			transcript += e.data;
		};
		eventSource.onerror = () => {
			// Close on error or completion
			if (eventSource) {
				eventSource.close();
				eventSource = null;
			}
			transcribing = false;
		};
	}

	function onDestroyCleanup() {
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
		if (audioContext) {
			audioContext.close();
		}
		if (eventSource) {
			eventSource.close();
		}
	}

	function getTxtPath(webmPath: string) {
		return webmPath.replace(/\.webm$/, '.txt');
	}

	async function summarizeTranscript() {
		if (!serverFilePath) return;
		summarizing = true;
		summary = 'Summarizing...';
		const txtPath = getTxtPath(serverFilePath);
		try {
			const res = await fetch(`/api/summarize?filepath=${encodeURIComponent(txtPath)}`);
			const data = await res.json();
			summary = data.summary;
		} catch (err) {
			summary = `Error summarizing: ${err instanceof Error ? err.message : 'Unknown error'}`;
		} finally {
			summarizing = false;
		}
	}

	// Cleanup on component destroy
	onDestroy(() => {
		onDestroyCleanup();
	});
</script>

<div class="recorder-controls">
	<button onclick={startRecording} disabled={startDisabled}>
		{recording ? 'Recording...' : 'Start Recording'}
	</button>
	<button onclick={stopRecording} disabled={stopDisabled}>Stop Recording</button>
</div>

{#if recording}
	<div class="mic-level-container">
		<label for="mic-level">Microphone Level:</label>
		<div class="mic-level-bar">
			<div
				class="mic-level-fill"
				style="width: {micLevel}%; background-color: {micLevel > 70
					? '#ff4444'
					: micLevel > 40
						? '#ffaa00'
						: '#44ff44'};"
			></div>
		</div>
		<span class="mic-level-text">{Math.round(micLevel)}%</span>
	</div>
{/if}

{#if audioURL}
	<div class="playback-section">
		<p>Playback:</p>
		<audio src={audioURL} controls></audio>
	</div>
{/if}

{#if uploadStatus}
	<p class="status">{uploadStatus}</p>
{/if}

{#if serverFilePath}
	<div class="server-info">
		<p><strong>Server filepath:</strong> {serverFilePath}</p>
	</div>
{/if}

{#if transcribing || transcript}
	<div class="transcription-section">
		<p><strong>Transcription:</strong></p>
		<pre
			class="transcription-text">{transcript}{#if transcribing}{'\n'}...transcribing...{/if}</pre>
	</div>
{/if}

{#if transcript && !transcribing}
	<!-- Allow speaker naming before summary -->
	<SpeakerNaming {audioURL} jsonPath={serverFilePath.replace(/\.webm$/, '.json')} />
	<button onclick={summarizeTranscript} disabled={summarizing} class="summarize-button">
		{summarizing ? 'Summarizing...' : 'Summarize Transcript'}
	</button>
{/if}

{#if summary}
	<div class="summary-section">
		<p><strong>Summary:</strong></p>
		<pre class="summary-text">{summary}</pre>
	</div>
	<!-- summary display -->
{/if}

<style>
	.recorder-controls {
		display: flex;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.recorder-controls button {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 8px;
		background-color: #4a90e2;
		color: white;
		font-weight: bold;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.recorder-controls button:hover:not(:disabled) {
		background-color: #357abd;
	}

	.recorder-controls button:disabled {
		background-color: #ccc;
		cursor: not-allowed;
	}

	.mic-level-container {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin: 1rem 0;
		padding: 1rem;
		background-color: #f8f9fa;
		border-radius: 8px;
		border: 2px solid #e9ecef;
	}

	.mic-level-bar {
		flex: 1;
		height: 20px;
		background-color: #e9ecef;
		border-radius: 10px;
		overflow: hidden;
		position: relative;
	}

	.mic-level-fill {
		height: 100%;
		transition: width 0.1s ease-out;
		border-radius: 10px;
	}

	.mic-level-text {
		font-weight: bold;
		min-width: 40px;
		text-align: right;
	}

	label {
		font-weight: bold;
		color: #495057;
	}

	.playback-section {
		margin: 1rem 0;
		padding: 1rem;
		background-color: #f8f9fa;
		border-radius: 8px;
	}

	.status {
		margin-top: 1rem;
		padding: 0.75rem;
		border-radius: 8px;
		background-color: #e3f2fd;
		border-left: 4px solid #2196f3;
		font-weight: 500;
	}

	.server-info {
		margin-top: 1rem;
		padding: 0.75rem;
		background-color: #e8f5e8;
		border-radius: 8px;
		border-left: 4px solid #4caf50;
	}

	.server-info p {
		margin: 0;
		font-family: 'Courier New', monospace;
		font-size: 0.9rem;
	}

	.transcription-section {
		margin: 1rem 0;
		padding: 1rem;
		background-color: #fff3cd;
		border-radius: 8px;
		border-left: 4px solid #ffc107;
	}

	.transcription-text {
		font-family: 'Courier New', monospace;
		font-size: 0.9rem;
		color: #856404;
		white-space: pre-wrap;
		word-wrap: break-word;
	}

	.summarize-button {
		margin-top: 1rem;
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 8px;
		background-color: #28a745;
		color: white;
		font-weight: bold;
		cursor: pointer;
	}

	.summarize-button:disabled {
		background-color: #ccc;
		cursor: not-allowed;
	}

	.summary-section {
		margin-top: 1rem;
		padding: 1rem;
		background-color: #e2e3e5;
		border-radius: 8px;
		border-left: 4px solid #6c757d;
	}

	.summary-text {
		font-family: 'Courier New', monospace;
		font-size: 0.9rem;
		white-space: pre-wrap;
		word-wrap: break-word;
	}
</style>
