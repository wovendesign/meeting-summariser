<script lang="ts">
  import SpeakerNaming from "$lib/components/SpeakerNaming.svelte";
  import Clipboard from "@lucide/svelte/icons/clipboard";
  import type { PageProps } from "./$types";

  import { page } from "$app/state";

  const meetingId = page.params.id;

  import { marked } from "marked";
  import * as Card from "$lib/components/ui/card/index.js";
  import Button, {
    buttonVariants,
  } from "$lib/components/ui/button/button.svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast, Toaster } from "svelte-sonner";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import RefreshCcw from "@lucide/svelte/icons/refresh-ccw";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import FolderClosed from "@lucide/svelte/icons/folder-closed";
  import Pen from "@lucide/svelte/icons/pen";
  import clsx from "clsx";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { listen, once } from "@tauri-apps/api/event";

  let transcriptContent = $state("");
  let transcriptJsonContent: string | null = $state(null);
  let summaryContent: string | null = $state("");
  let savingTranscript = $state(false);
  let saveStatus = $state("");
  let loadingSummary = $state(false);

  let name = $state(`Meeting ${meetingId}`);
  let generatingName = $state(false);

  let isTranscribing: String | null = $state(null);
  let isSummarizing: String | null = $state(null);

  // Transcription progress tracking
  let transcriptionProgress = $state({
    currentChunk: 0,
    totalChunks: 0,
    isActive: false,
  });

  // Audio splitting progress tracking
  let audioSplittingProgress = $state({
    currentChunk: 0,
    totalChunks: 0,
    isActive: false,
  });

  import { readFile, BaseDirectory } from "@tauri-apps/plugin-fs";

  let meetingMetadata: {
    name?: string;
  } = $state({});

  $effect(() => {
    if (meetingMetadata.name) {
      name = meetingMetadata.name;
    }
  });

  let audio: Uint8Array | null = $state(null);
  let audioURL = $derived.by(() => {
    if (!audio) return "";
    console.log(audio);
    const blob = new Blob([audio], { type: "audio/ogg" });
    return URL.createObjectURL(blob);
  });

  async function saveTranscript() {
    // savingTranscript = true;
    // saveStatus = '';
    // try {
    // 	const res = await fetch('/api/transcript/update', {
    // 		method: 'POST',
    // 		headers: { 'Content-Type': 'application/json' },
    // 		body: JSON.stringify({ filepath: data.txtPath, content: transcriptContent })
    // 	});
    // 	saveStatus = res.ok ? 'Transcript saved' : `Save failed: ${res.statusText}`;
    // } catch (err) {
    // 	saveStatus = `Error: ${err instanceof Error ? err.message : err}`;
    // } finally {
    // 	savingTranscript = false;
    // }
  }

  async function regenerateSummary() {
    try {
      summaryContent = await invoke("generate_summary", {
        meetingId: meetingId,
      });
      await getMeetingMetadata();
      isSummarizing = null; // Reset summarization status after fetching summary
    } catch (error) {
      console.error("Error regenerating summary:", error);
      // Handle error appropriately, e.g., show a toast notification
      toast.error("Error regenerating summary: " + error);
      saveStatus = "Error regenerating summary";
    }
  }

  const markdownContent = $derived.by(() => {
    if (!summaryContent) return "";
    const realNewLinesContent = summaryContent.replace(/\n/g, "\n");
    return marked(realNewLinesContent, {
      gfm: true,
      breaks: true,
    });
  });

  onMount(async () => {
    // Listen for whisperx progress events
    const whisperxStartListener = await listen<number>(
      "whisperx-start",
      (event) => {
        console.log("WhisperX started with", event.payload, "chunks");
        transcriptionProgress.totalChunks = event.payload;
        transcriptionProgress.currentChunk = 0;
        transcriptionProgress.isActive = true;
        audioSplittingProgress.isActive = false; // Audio splitting is done
      },
    );

    const whisperxProgressListener = await listen<number>(
      "whisperx-progress",
      (event) => {
        console.log(
          "WhisperX progress:",
          event.payload + 1,
          "of",
          transcriptionProgress.totalChunks,
        );
        transcriptionProgress.currentChunk = event.payload + 1; // +1 because backend sends 0-based index
      },
    );

    // Listen for ffmpeg progress events
    const ffmpegStartListener = await listen<number>(
      "ffmpeg-start",
      (event) => {
        console.log("FFmpeg started with", event.payload, "chunks");
        audioSplittingProgress.totalChunks = event.payload;
        audioSplittingProgress.currentChunk = 0;
        audioSplittingProgress.isActive = true;
      },
    );

    const ffmpegProgressListener = await listen<number>(
      "ffmpeg-progress",
      (event) => {
        console.log(
          "FFmpeg progress:",
          event.payload + 1,
          "of",
          audioSplittingProgress.totalChunks,
        );
        audioSplittingProgress.currentChunk = event.payload + 1; // +1 because backend sends 0-based index
      },
    );

    // Store the listeners for cleanup
    whisperxListeners = [
      whisperxStartListener,
      whisperxProgressListener,
      ffmpegStartListener,
      ffmpegProgressListener,
    ];

    isTranscribing = await invoke("is_transcribing", {
      meetingId: meetingId,
    });

    isSummarizing = await invoke("is_summarizing", {
      meetingId: meetingId,
    });

    if (!isTranscribing) {
      await getTranscript();
      await getTranscriptJson();
    }
    if (!isSummarizing) {
      await getSummary();
    }
    await getAudio();
    await getMeetingMetadata();

    // register Tauri event listeners once on mount
    summarizationListener = await once<string>(
      "summarization-started",
      (event) => {
        console.log(event);
        isSummarizing = event.payload;
        console.log("Summarization started for meeting ID:", event.payload);
        toast.info("Summarization started: " + event.payload);
      },
    );
    transcriptionListener = await listen<string>(meetingId, (event) => {
      if (event.payload === "transcription-started") {
        toast.info("Transcription started for meeting ID: " + meetingId);
        isTranscribing = meetingId;
        transcriptionProgress.isActive = true;
        audioSplittingProgress.isActive = true; // Audio splitting happens first
      } else if (event.payload === "transcription-finished") {
        toast.success("Transcription finished for meeting ID: " + meetingId);
        isTranscribing = null;
        transcriptionProgress.isActive = false;
        transcriptionProgress.currentChunk = 0;
        transcriptionProgress.totalChunks = 0;
        audioSplittingProgress.isActive = false;
        audioSplittingProgress.currentChunk = 0;
        audioSplittingProgress.totalChunks = 0;
        getTranscript();
        getTranscriptJson();
        getSummary(true);
      }
    });
  });

  onDestroy(() => {
    summarizationListener();
    transcriptionListener();
    // Clean up whisperx listeners
    whisperxListeners.forEach((listener) => listener());
  });

  async function checkTranscriptionStatus() {
    try {
      isTranscribing = await invoke("is_transcribing", {
        meetingId: meetingId,
      });
      console.log(
        "Transcription status for meeting ID",
        meetingId,
        ":",
        isTranscribing,
      );
    } catch (error) {
      toast.error("Error checking transcription status: " + error);
      console.error("Error checking transcription status:", error);
    }
  }

  async function getTranscript() {
    try {
      transcriptContent = await invoke("get_meeting_transcript", {
        meetingId: meetingId,
      });
      isTranscribing = null; // Reset transcription status after fetching transcript
    } catch (error) {
      console.error("Error fetching transcript:", error);

      await checkTranscriptionStatus();
      if (isTranscribing && isTranscribing !== meetingId) {
        // If transcription is still in progress, show a toast notification
        toast.info("Another Transcription is still in progress. Please wait.");
        return;
      } else if (isTranscribing === meetingId) {
        // If transcription is in progress for this meeting, show a toast notification
        toast.info("Transcription is still in progress. Please wait.");
        return;
      }

      // When the transcribe function settles without an error, retry fetching the transcript, otherwise show a toast error
      invoke("transcribe_with_chunking", { meetingId: meetingId })
        .then(() => {
          console.log("Transcription finished successfully");
          getTranscript();
          getTranscriptJson();

          // Load Summary, and generate one if it doesn't exist
          getSummary(true);
        })
        .catch((err) => {
          console.error("Error transcribing audio:", err);
        });

      await checkTranscriptionStatus();
    }
  }

  async function transcribe() {
    invoke("transcribe_with_chunking", { meetingId: meetingId })
      .then(() => {
        getTranscript();
        getTranscriptJson();
        getSummary(true);
      })
      .catch((error) => {
        console.error("Error starting transcription:", error);
        toast.error("Error starting transcription: " + error);
      });
    checkTranscriptionStatus();
  }

  async function getTranscriptJson() {
    try {
      transcriptJsonContent = await invoke("get_meeting_transcript_json", {
        meetingId: meetingId,
      });
    } catch (error) {
      console.error("Error fetching transcript:", error);
      transcriptJsonContent = null;
    }
  }

  async function getSummary(generateIfNotExists = false) {
    try {
      console.log("Fetching summary for meeting ID:", meetingId);
      summaryContent = await invoke("get_meeting_summary", {
        meetingId: meetingId,
      });
    } catch (error) {
      console.error("Error fetching summary:", error);
      summaryContent = null;

      if (generateIfNotExists) {
        await regenerateSummary();
      } else {
        toast.error("Error fetching summary: " + error);
        return;
      }
    }
  }

  async function getAudio() {
    try {
      // audio = await invoke("get_meeting_audio", { meetingId: meetingId });
      const icon = await readFile(`uploads/${meetingId}/${meetingId}.ogg`, {
        baseDir: BaseDirectory.AppLocalData,
      });
      audio = icon;
    } catch (error) {
      console.error("Error fetching audio:", error);
      audio = null;
    }
  }

  async function getMeetingMetadata() {
    try {
      meetingMetadata = await invoke("get_meeting_metadata", {
        meetingId: meetingId,
      });
    } catch (error) {
      console.error("Error fetching meeting metadata:", error);
      meetingMetadata = {};
    }
  }

  async function reloadTranscript() {
    try {
      await getTranscript();
      await getTranscriptJson();
      toast.success("Transcript reloaded successfully!");
    } catch (error) {
      console.error("Error reloading transcript:", error);
      toast.error("Error reloading transcript: " + error);
    }
  }

  let summarizationListener: () => void;
  let transcriptionListener: () => void;
  let whisperxListeners: (() => void)[] = [];
</script>

<Toaster />
<div class="flex flex-col gap-4 p-8 overflow-y-scroll h-full">
  <Button variant="outline" href="/" class="self-start">Back</Button>
  <div class="flex items-center justify-between">
    <h2 class={clsx("text-2xl font-bold", generatingName && "animate-pulse")}>
      {name}
    </h2>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger
        class={buttonVariants({ variant: "outline", size: "icon" })}
      >
        <Ellipsis />
      </DropdownMenu.Trigger>
      <DropdownMenu.Content class="w-56 mr-4">
        <DropdownMenu.Item>
          <Pen />
          <span>Rename Meeting</span>
          <Dialog.Root>
            <Dialog.Trigger class={buttonVariants({ variant: "outline" })}
              >Edit Profile</Dialog.Trigger
            >
            <Dialog.Content class="sm:max-w-[425px]">
              <Dialog.Header>
                <Dialog.Title>Edit profile</Dialog.Title>
                <Dialog.Description>
                  Make changes to your profile here. Click save when you're
                  done.
                </Dialog.Description>
              </Dialog.Header>
              <div class="grid gap-4 py-4">
                <div class="grid grid-cols-4 items-center gap-4">
                  <Label for="name" class="text-right">Name</Label>
                  <Input id="name" value="Pedro Duarte" class="col-span-3" />
                </div>
                <div class="grid grid-cols-4 items-center gap-4">
                  <Label for="username" class="text-right">Username</Label>
                  <Input id="username" value="@peduarte" class="col-span-3" />
                </div>
              </div>
              <Dialog.Footer>
                <Button type="submit">Save changes</Button>
              </Dialog.Footer>
            </Dialog.Content>
          </Dialog.Root>
        </DropdownMenu.Item>
        <!-- <DropdownMenu.Item onclick={generateMeetingName}>
          <RefreshCcw />
          <span>Re-Generate Name</span>
        </DropdownMenu.Item> -->
        <DropdownMenu.Item
          onclick={() => {
            alert("This feature is not implemented yet.");
          }}
        >
          <FolderClosed />
          <span>Reveal in Finder</span>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>

  <Card.Root class="group">
    <Card.Header class="flex items-center justify-between">
      <Card.Title>Recorded Meeting</Card.Title>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger
          class={clsx(buttonVariants({ variant: "outline", size: "icon" }))}
        >
          <Ellipsis />
        </DropdownMenu.Trigger>
        <DropdownMenu.Content class="w-56 mr-4">
          <DropdownMenu.Item onclick={transcribe}>
            <RefreshCcw />
            <span>Transcribe Audio</span>
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </Card.Header>
    <Card.Content>
      <audio src={audioURL} controls class="w-full"></audio>
    </Card.Content>
  </Card.Root>

  <section>
    {#if saveStatus}
      <p>{saveStatus}</p>
    {/if}

    <Card.Root>
      <Card.Header>
        <Card.Title>Transcript</Card.Title>
      </Card.Header>
      <Card.Content>
        {#if isTranscribing === meetingId}
          <div class="space-y-4">
            {#if audioSplittingProgress.isActive && audioSplittingProgress.totalChunks > 0}
              <!-- Audio splitting progress -->
              <div class="space-y-2">
                <div class="flex justify-between text-sm">
                  <span
                    >Splitting Audio Chunk {audioSplittingProgress.currentChunk}
                    of {audioSplittingProgress.totalChunks}</span
                  >
                  <span
                    >{Math.round(
                      (audioSplittingProgress.currentChunk /
                        audioSplittingProgress.totalChunks) *
                        100,
                    )}%</span
                  >
                </div>
                <div class="w-full bg-muted rounded-full h-2">
                  <div
                    class="bg-orange-500 h-2 rounded-full transition-all duration-300 ease-in-out"
                    style="width: {(audioSplittingProgress.currentChunk /
                      audioSplittingProgress.totalChunks) *
                      100}%"
                  ></div>
                </div>
              </div>
            {/if}
            {#if transcriptionProgress.isActive && transcriptionProgress.totalChunks > 0}
              <!-- Transcription progress -->
              <div class="space-y-2">
                <div class="flex justify-between text-sm">
                  <span
                    >Transcribing Chunk {transcriptionProgress.currentChunk} of {transcriptionProgress.totalChunks}</span
                  >
                  <span
                    >{Math.round(
                      (transcriptionProgress.currentChunk /
                        transcriptionProgress.totalChunks) *
                        100,
                    )}%</span
                  >
                </div>
                <div class="w-full bg-muted rounded-full h-2">
                  <div
                    class="bg-primary h-2 rounded-full transition-all duration-300 ease-in-out"
                    style="width: {(transcriptionProgress.currentChunk /
                      transcriptionProgress.totalChunks) *
                      100}%"
                  ></div>
                </div>
              </div>
            {/if}
            <!-- Animated placeholder -->
            <div class={"flex-wrap gap-2 animate-pulse flex"}>
              <div class="h-4 w-12 bg-foreground/10 rounded"></div>
              <div class="h-4 w-24 bg-foreground/10 rounded"></div>
              <div class="h-4 w-16 bg-foreground/10 rounded"></div>
              <div class="h-4 w-20 bg-foreground/10 rounded"></div>
              <div class="h-4 w-32 bg-foreground/10 rounded"></div>
              <div class="h-4 w-28 bg-foreground/10 rounded"></div>
              <div class="h-4 w-24 bg-foreground/10 rounded"></div>
              <div class="h-4 w-20 bg-foreground/10 rounded"></div>
              <div class="h-4 w-16 bg-foreground/10 rounded"></div>
              <div class="h-4 w-12 bg-foreground/10 rounded"></div>
              <div class="h-4 w-24 bg-foreground/10 rounded"></div>
              <div class="h-4 w-16 bg-foreground/10 rounded"></div>
              <div class="h-4 w-20 bg-foreground/10 rounded"></div>
              <div class="h-4 w-32 bg-foreground/10 rounded"></div>
              <div class="h-4 w-28 bg-foreground/10 rounded"></div>
              <div class="h-4 w-24 bg-foreground/10 rounded"></div>
            </div>
          </div>
        {:else if isTranscribing && isTranscribing !== meetingId}
          <p class="text-sm text-muted-foreground">
            Another transcription is in progress. Please wait.
          </p>
        {:else if transcriptContent}
          <Textarea
            bind:value={transcriptContent}
            placeholder="Edit the Transcript"
          />
        {/if}
      </Card.Content>
      <Card.Footer class="flex gap-2">
        {#if !isTranscribing}
          <Button onclick={saveTranscript} disabled={savingTranscript}>
            {savingTranscript ? "Saving..." : "Save Transcript"}
          </Button>
        {/if}
      </Card.Footer>
    </Card.Root>
  </section>

  <section>
    <SpeakerNaming
      {audioURL}
      {reloadTranscript}
      {meetingId}
      json={transcriptJsonContent}
    />
  </section>

  <section>
    <Card.Root>
      <Card.Header>
        <Card.Title>Transcription Summary</Card.Title>
      </Card.Header>
      <Card.Content class="prose prose-invert mx-auto">
        {#if isSummarizing === meetingId}
          <div class={"flex-wrap gap-2 animate-pulse flex"}>
            <div class="h-4 w-12 bg-foreground/10 rounded"></div>
            <div class="h-4 w-24 bg-foreground/10 rounded"></div>
            <div class="h-4 w-16 bg-foreground/10 rounded"></div>
            <div class="h-4 w-20 bg-foreground/10 rounded"></div>
            <div class="h-4 w-32 bg-foreground/10 rounded"></div>
            <div class="h-4 w-28 bg-foreground/10 rounded"></div>
            <div class="h-4 w-24 bg-foreground/10 rounded"></div>
            <div class="h-4 w-20 bg-foreground/10 rounded"></div>
            <div class="h-4 w-16 bg-foreground/10 rounded"></div>
            <div class="h-4 w-12 bg-foreground/10 rounded"></div>
            <div class="h-4 w-24 bg-foreground/10 rounded"></div>
            <div class="h-4 w-16 bg-foreground/10 rounded"></div>
            <div class="h-4 w-20 bg-foreground/10 rounded"></div>
            <div class="h-4 w-32 bg-foreground/10 rounded"></div>
            <div class="h-4 w-28 bg-foreground/10 rounded"></div>
            <div class="h-4 w-24 bg-foreground/10 rounded"></div>
          </div>
        {:else if isSummarizing && isSummarizing !== meetingId && !summaryContent}
          <p class="text-sm text-muted-foreground">
            Another summarization is in progress. Please wait.
          </p>
        {:else if summaryContent}
          {@html markdownContent}
        {:else}
          <p>No summary available.</p>
        {/if}
      </Card.Content>
      <Card.Footer class="flex gap-2">
        <Button>
          <Clipboard class="mr-2 size-4" />
          Copy Summary
        </Button>
        <Button onclick={regenerateSummary} disabled={loadingSummary}>
          {loadingSummary ? "Regenerating..." : "Regenerate Summary"}
        </Button>
      </Card.Footer>
    </Card.Root>
  </section>
</div>
