<script lang="ts">
  import { writeFile, BaseDirectory, mkdir } from "@tauri-apps/plugin-fs";
  import { onDestroy } from "svelte";
  import { Button } from "./ui/button";
  import Mic from "@lucide/svelte/icons/mic";
  import CircleStop from "@lucide/svelte/icons/circle-stop";

  // State runes
  let mediaRecorder = $state<MediaRecorder | null>(null);
  let audioChunks = $state<Blob[]>([]);
  let recording = $state(false);
  let audioURL = $state("");
  let uploadStatus = $state("");

  let isUploading = $state(false);
  let micLevel = $state(0);
  let eventSource: EventSource | null = null;

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
      if (audioContext.state === "suspended") {
        await audioContext.resume();
      }

      const source = audioContext.createMediaStreamSource(stream);
      analyser = audioContext.createAnalyser();
      analyser.fftSize = 2048; // Increase for better resolution
      analyser.smoothingTimeConstant = 0.3; // Smooth out the values
      source.connect(analyser);

      console.log("Audio context state:", audioContext.state);
      console.log("Analyser setup complete");

      mediaRecorder = new MediaRecorder(stream);

      mediaRecorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: "audio/ogg" });
        audioURL = URL.createObjectURL(audioBlob);
        audioChunks = [];

        // Clean up audio analysis
        if (animationFrame) {
          cancelAnimationFrame(animationFrame);
          animationFrame = null;
        }
        if (audioContext && audioContext.state !== "closed") {
          audioContext.close();
        }
        audioContext = null;
        analyser = null;
        micLevel = 0;

        // Safe Recording
        const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
        const recordingName = `recording-${timestamp}`;

        const contents = new Uint8Array(await audioBlob.arrayBuffer());
        // Create Meeting Directory
        await mkdir(`uploads/${recordingName}`, {
          baseDir: BaseDirectory.AppLocalData,
        });
        // Write the recording file
        await writeFile(
          `uploads/${recordingName}/${recordingName}.ogg`,
          contents,
          {
            baseDir: BaseDirectory.AppLocalData,
          },
        );

        // Redirect to new meeting page
        window.location.href = `/meeting/${recordingName}`;
      };

      mediaRecorder.start();
      recording = true;

      // Start analyzing audio levels AFTER setting recording to true
      updateMicLevel();
    } catch (error) {
      console.error("Error starting recording:", error);
      uploadStatus = `Error starting recording: ${error instanceof Error ? error.message : "Unknown error"}`;
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
      console.log("No analyser or not recording");
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
      console.log("RMS:", rms, "Level:", newLevel);
    }

    if (recording) {
      animationFrame = requestAnimationFrame(updateMicLevel);
    }
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

  // Cleanup on component destroy
  onDestroy(() => {
    onDestroyCleanup();
  });
</script>

{#if recording}
  <div class="flex items-center gap-4">
    <Button onclick={stopRecording} disabled={stopDisabled}>
      <CircleStop />
      Stop Recording
    </Button>
    <div
      class="w-full h-2 rounded-full bg-foreground/10 max-w-24 overflow-clip"
    >
      <div
        class="bg-foreground h-full rounded-full"
        style="width: {micLevel}%;"
      ></div>
    </div>
  </div>
{:else}
  <Button
    class="self-start bg-red-500 text-white hover:bg-red-600"
    onclick={startRecording}
    disabled={startDisabled}
  >
    <Mic />
    Start Recording
  </Button>
{/if}
