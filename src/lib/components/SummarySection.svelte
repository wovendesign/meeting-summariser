<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import Clipboard from "@lucide/svelte/icons/clipboard";
  import ProgressBar from "./ProgressBar.svelte";
  import LoadingPlaceholder from "./LoadingPlaceholder.svelte";

  interface ProgressState {
    currentChunk: number;
    totalChunks: number;
    isActive: boolean;
  }

  interface Props {
    summaryContent: string | null;
    markdownContent: string;
    isSummarizing: string | null;
    meetingId: string;
    summarizationProgress: ProgressState;
    loadingSummary: boolean;
    onRegenerateSummary?: () => void;
    onCopySummary?: () => void;
  }

  let {
    summaryContent,
    markdownContent,
    isSummarizing,
    meetingId,
    summarizationProgress,
    loadingSummary,
    onRegenerateSummary,
    onCopySummary,
  }: Props = $props();

  // Adjusted percentage calculation to match the original logic
  const adjustedPercentage = $derived(() => {
    if (summarizationProgress.totalChunks === 0) return 0;
    return Math.round(
      ((summarizationProgress.currentChunk - 1) /
        summarizationProgress.totalChunks) *
        100,
    );
  });
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Transcription Summary</Card.Title>
  </Card.Header>
  <Card.Content class="prose prose-invert mx-auto">
    {#if isSummarizing === meetingId}
      <div class="space-y-4">
        {#if summarizationProgress.isActive && summarizationProgress.totalChunks > 0}
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span>
                Summarizing Chunk {summarizationProgress.currentChunk} of {summarizationProgress.totalChunks}
              </span>
              <span>{adjustedPercentage()}%</span>
            </div>
            <div class="w-full bg-muted rounded-full h-2">
              <div
                class="bg-primary h-2 rounded-full transition-all duration-300 ease-in-out"
                style="width: {adjustedPercentage()}%"
              ></div>
            </div>
          </div>
        {/if}
        <LoadingPlaceholder />
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
    <Button onclick={onCopySummary}>
      <Clipboard class="mr-2 size-4" />
      Copy Summary
    </Button>
    <Button onclick={onRegenerateSummary} disabled={loadingSummary}>
      {loadingSummary ? "Regenerating..." : "Regenerate Summary"}
    </Button>
  </Card.Footer>
</Card.Root>
