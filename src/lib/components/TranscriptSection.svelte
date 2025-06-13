<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import { Textarea } from "$lib/components/ui/textarea";
  import ProgressBar from "./ProgressBar.svelte";
  import LoadingPlaceholder from "./LoadingPlaceholder.svelte";

  interface ProgressState {
    currentChunk: number;
    totalChunks: number;
    isActive: boolean;
  }

  interface Props {
    transcriptContent: string;
    isTranscribing: string | null;
    meetingId: string;
    audioSplittingProgress: ProgressState;
    transcriptionProgress: ProgressState;
    savingTranscript: boolean;
    onSaveTranscript?: () => void;
    onTranscriptChange?: (value: string) => void;
  }

  let {
    transcriptContent = $bindable(),
    isTranscribing,
    meetingId,
    audioSplittingProgress,
    transcriptionProgress,
    savingTranscript,
    onSaveTranscript,
  }: Props = $props();
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Transcript</Card.Title>
  </Card.Header>
  <Card.Content>
    {#if isTranscribing === meetingId}
      <div class="space-y-4">
        {#if audioSplittingProgress.isActive && audioSplittingProgress.totalChunks > 0}
          <ProgressBar
            currentChunk={audioSplittingProgress.currentChunk}
            totalChunks={audioSplittingProgress.totalChunks}
            label="Splitting Audio Chunk"
            color="bg-orange-500"
          />
        {/if}
        {#if transcriptionProgress.isActive && transcriptionProgress.totalChunks > 0}
          <ProgressBar
            currentChunk={transcriptionProgress.currentChunk}
            totalChunks={transcriptionProgress.totalChunks}
            label="Transcribing Chunk"
          />
        {/if}
        <LoadingPlaceholder />
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
      <Button onclick={onSaveTranscript} disabled={savingTranscript}>
        {savingTranscript ? "Saving..." : "Save Transcript"}
      </Button>
    {/if}
  </Card.Footer>
</Card.Root>
