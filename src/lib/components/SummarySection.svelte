<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import Clipboard from "@lucide/svelte/icons/clipboard";
  import ProgressBar from "./ProgressBar.svelte";
  import LoadingPlaceholder from "./LoadingPlaceholder.svelte";
  import { marked } from "marked";

  interface ProgressState {
    currentChunk: number;
    totalChunks: number;
    isActive: boolean;
  }

  interface ChunkSummary {
    chunk_number: number;
    content: string;
    markdown_content: string;
  }

  interface Props {
    summaryContent: string | null;
    markdownContent: string;
    chunkSummaries?: ChunkSummary[];
    isSummarizing: string | null;
    meetingId: string;
    summarizationProgress: ProgressState;
    loadingSummary: boolean;
    onRegenerateSummary?: () => void;
    onRegenerateFinalSummary?: () => void;
    onCopySummary?: () => void;
  }

  let {
    summaryContent,
    markdownContent,
    chunkSummaries = [],
    isSummarizing,
    meetingId,
    summarizationProgress,
    loadingSummary,
    onRegenerateSummary,
    onRegenerateFinalSummary,
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

  // Tab state
  let activeTab = $state("final-summary");
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Transcription Summary</Card.Title>
  </Card.Header>
  <Card.Content>
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
      <Tabs.Root bind:value={activeTab}>
        <Tabs.List class="grid w-full grid-cols-2">
          <Tabs.Trigger value="final-summary">Final Summary</Tabs.Trigger>
          <Tabs.Trigger value="chunk-summaries">Chunk Summaries</Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content
          value="final-summary"
          class="prose prose-invert mx-auto mt-4"
        >
          {@html markdownContent}
        </Tabs.Content>

        <Tabs.Content value="chunk-summaries" class="mt-4">
          {#if chunkSummaries && chunkSummaries.length > 0}
            <div class="space-y-4">
              {#each chunkSummaries as chunk}
                <div class="border border-border rounded-md p-4">
                  <h4 class="text-sm font-medium mb-2 text-muted-foreground">
                    Chunk {chunk.chunk_number}
                  </h4>
                  <div class="prose prose-invert prose-sm">
                    {@html marked(chunk.markdown_content, {
                      gfm: true,
                      breaks: true,
                    })}
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-sm text-muted-foreground">
              No chunk summaries available.
            </p>
          {/if}
        </Tabs.Content>
      </Tabs.Root>
    {:else}
      <p>No summary available.</p>
    {/if}
  </Card.Content>
  <Card.Footer class="flex gap-2">
    <Button onclick={onCopySummary}>
      <Clipboard class="mr-2 size-4" />
      Copy Summary
    </Button>
    <Button
      onclick={onRegenerateFinalSummary}
      disabled={loadingSummary}
      variant="outline"
    >
      {loadingSummary ? "Regenerating..." : "Regenerate Final Summary"}
    </Button>
    <Button onclick={onRegenerateSummary} disabled={loadingSummary}>
      {loadingSummary ? "Regenerating..." : "Regenerate Full Summary"}
    </Button>
  </Card.Footer>
</Card.Root>
