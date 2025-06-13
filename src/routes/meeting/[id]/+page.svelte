<script lang="ts">
  import SpeakerNaming from "$lib/components/SpeakerNaming.svelte";
  import MeetingHeader from "$lib/components/MeetingHeader.svelte";
  import AudioPlayer from "$lib/components/AudioPlayer.svelte";
  import TranscriptSection from "$lib/components/TranscriptSection.svelte";
  import SummarySection from "$lib/components/SummarySection.svelte";
  import type { PageProps } from "./$types";

  import { page } from "$app/state";
  import { marked } from "marked";
  import Button from "$lib/components/ui/button/button.svelte";
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast, Toaster } from "svelte-sonner";

  import { useMeetingData } from "$lib/hooks/useMeetingData.svelte";
  import { useProgressTracking } from "$lib/hooks/useProgressTracking.svelte";
  import { useEventListeners } from "$lib/hooks/useEventListeners";
  import { useTranscriptManager } from "$lib/hooks/useTranscriptManager";

  const meetingId = page.params.id;

  // Initialize hooks
  const meetingData = useMeetingData(meetingId);
  const progressTracking = useProgressTracking();
  const transcriptManager = useTranscriptManager(meetingId, meetingData);

  // Local state
  let savingTranscript = $state(false);
  let saveStatus = $state("");
  let loadingSummary = $state(false);
  let name = $state(`Meeting ${meetingId}`);
  let generatingName = $state(false);
  let isTranscribing: string | null = $state(null);
  let isSummarizing: string | null = $state(null);

  // Derived values
  const meetingMetadata = $derived(meetingData.meetingMetadata);
  const summaryContent = $derived(meetingData.summaryContent);
  const audioURL = $derived(meetingData.audioURL);

  // Update name when metadata changes
  $effect(() => {
    if (meetingMetadata.name) {
      name = meetingMetadata.name;
    }
  });

  const markdownContent = $derived.by(() => {
    if (!summaryContent) return "";
    const realNewLinesContent = summaryContent.replace(/\n/g, "\n");
    const result = marked(realNewLinesContent, {
      gfm: true,
      breaks: true,
    });
    // Handle both sync and async results from marked
    return typeof result === "string" ? result : "";
  });

  // Event handlers
  async function saveTranscript() {
    // Implementation remains the same - placeholder for now
  }

  async function checkTranscriptionStatus() {
    return await transcriptManager.checkTranscriptionStatus();
  }

  async function handleTranscriptionStarted() {
    isTranscribing = meetingId;
    progressTracking.resetAudioSplittingProgress();
    progressTracking.resetTranscriptionProgress();
  }

  async function handleTranscriptionFinished() {
    isTranscribing = null;
    progressTracking.resetTranscriptionProgress();
    progressTracking.resetAudioSplittingProgress();

    await meetingData.getTranscript();
    await meetingData.getTranscriptJson();
    await meetingData.getSummary();
  }

  async function handleSummarizationStarted(summaryMeetingId: string) {
    isSummarizing = summaryMeetingId;
    progressTracking.resetSummarizationProgress();
  }

  async function handleRegenerateSummary() {
    try {
      await meetingData.regenerateSummary();
      await meetingData.getMeetingMetadata();
      isSummarizing = null;
      progressTracking.resetSummarizationProgress();
    } catch (error) {
      console.error("Error regenerating summary:", error);
      saveStatus = "Error regenerating summary";
      progressTracking.resetSummarizationProgress();
    }
  }

  async function handleRevealInFinder() {
    alert("This feature is not implemented yet.");
  }

  async function handleRenameMeeting(newName: string) {
    try {
      generatingName = true;
      await invoke("rename_meeting", { meetingId, newName });
      name = newName;
      // Reload metadata to reflect the change
      await meetingData.getMeetingMetadata();
      toast.success("Meeting renamed successfully!");
    } catch (error) {
      console.error("Error renaming meeting:", error);
      toast.error("Failed to rename meeting");
    } finally {
      generatingName = false;
    }
  }

  async function handleCopySummary() {
    if (summaryContent) {
      await navigator.clipboard.writeText(summaryContent);
      toast.success("Summary copied to clipboard!");
    }
  }

  // Initialize event listeners
  const eventListeners = useEventListeners(meetingId, progressTracking, {
    onTranscriptionStarted: handleTranscriptionStarted,
    onTranscriptionFinished: handleTranscriptionFinished,
    onSummarizationStarted: handleSummarizationStarted,
  });

  onMount(async () => {
    // Setup event listeners
    await eventListeners.setupListeners();

    // Check initial status
    isTranscribing = await invoke("is_transcribing", { meetingId });
    isSummarizing = await invoke("is_summarizing", { meetingId });

    // Load data if not processing
    if (!isTranscribing) {
      await transcriptManager.handleTranscriptLoad();
      await meetingData.getTranscriptJson();
    }
    if (!isSummarizing) {
      await meetingData.getSummary();
    }

    await meetingData.getAudio();
    await meetingData.getMeetingMetadata();
  });

  onDestroy(() => {
    eventListeners.cleanup();
  });
</script>

<Toaster />
<div class="flex flex-col gap-4 p-8 overflow-y-scroll h-full">
  <Button variant="outline" href="/" class="self-start">Back</Button>

  <MeetingHeader
    {name}
    {generatingName}
    onRevealInFinder={handleRevealInFinder}
    onRenameMeeting={handleRenameMeeting}
  />

  <AudioPlayer {audioURL} onTranscribe={meetingData.transcribe} />

  <section>
    {#if saveStatus}
      <p>{saveStatus}</p>
    {/if}

    <TranscriptSection
      bind:transcriptContent={meetingData.transcriptContent}
      {isTranscribing}
      {meetingId}
      audioSplittingProgress={progressTracking.audioSplittingProgress}
      transcriptionProgress={progressTracking.transcriptionProgress}
      {savingTranscript}
      onSaveTranscript={saveTranscript}
    />
  </section>

  <section>
    <SpeakerNaming
      {audioURL}
      reloadTranscript={transcriptManager.reloadTranscript}
      {meetingId}
      json={meetingData.transcriptJsonContent}
    />
  </section>

  <section>
    <SummarySection
      {summaryContent}
      {markdownContent}
      {isSummarizing}
      {meetingId}
      summarizationProgress={progressTracking.summarizationProgress}
      {loadingSummary}
      onRegenerateSummary={handleRegenerateSummary}
      onCopySummary={handleCopySummary}
    />
  </section>
</div>
