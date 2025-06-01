<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import * as Card from "$lib/components/ui/card/index.js";
  import Play from "@lucide/svelte/icons/play";
  import Pause from "@lucide/svelte/icons/pause";
  import { Input } from "$lib/components/ui/input";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";

  const {
    meetingId,
    audioURL,
    json,
    reloadTranscript,
  }: {
    meetingId: string;
    audioURL: string;
    json: string | null;
    reloadTranscript: () => void;
  } = $props();
  interface Segment {
    start: number;
    end: number;
    speaker: string;
  }

  let speakerNames: Record<string, string> = $state({});
  let audioElem!: HTMLAudioElement;
  let playQueue: Segment[] = [];
  let queueIndex = 0;
  let saving = $state(false);
  let saveStatus = "";
  let currentlyPlayingSpeaker: string | null = $state(null);

  let segments: Segment[] = $derived.by(() => {
    if (!json) return [];
    const data = JSON.parse(json);
    return data.segments || [];
  });

  // let speakers: string[] = $state([]);
  let speakers: string[] = $derived.by(() => {
    if (!segments) return [];
    const set = new Set<string>(segments.map((s: Segment) => s.speaker));
    return Array.from(set);
  });

  $effect(() => {
    speakers.forEach((id) => {
      if (!speakerNames[id]) {
        speakerNames[id] = id; // Initialize with speaker ID
      }
    });
  });

  function playSpeaker(id: string) {
    if (currentlyPlayingSpeaker === id) {
      audioElem.pause();
      currentlyPlayingSpeaker = null;
      return;
    }
    playQueue = segments
      .filter((s) => s.speaker === id)
      .sort((a, b) => a.start - b.start);
    if (!playQueue.length) return;
    queueIndex = 0;
    audioElem.currentTime = playQueue[0].start;
    audioElem.play();
    currentlyPlayingSpeaker = id;
  }

  function onTimeUpdate() {
    if (!playQueue.length) return;
    const segment = playQueue[queueIndex];
    if (audioElem.currentTime >= segment.end) {
      queueIndex++;
      if (queueIndex < playQueue.length) {
        audioElem.currentTime = playQueue[queueIndex].start;
      } else {
        audioElem.pause();
      }
    }
  }

  async function saveNames() {
    console.log("Saving speaker names:", speakerNames);
    try {
      await invoke("save_speaker_names", { meetingId, names: speakerNames });
      reloadTranscript();
      toast.success("Speaker names saved successfully!");
    } catch (error) {
      console.error("Error saving speaker names:", error);
      toast.error(
        `Error saving speaker names: ${error instanceof Error ? error.message : error}`,
      );
    }
    // saving = true;
    // saveStatus = '';
    // try {
    // 	const res = await fetch('/api/segments/update', {
    // 		method: 'POST',
    // 		headers: { 'Content-Type': 'application/json' },
    // 		body: JSON.stringify({ jsonPath, speakerNames })
    // 	});
    // 	saveStatus = res.ok ? 'Speaker names saved' : `Save failed: ${res.statusText}`;
    // } catch (err) {
    // 	saveStatus = `Error: ${err instanceof Error ? err.message : err}`;
    // } finally {
    // 	saving = false;
    // }
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Speaker Naming</Card.Title>
  </Card.Header>
  <Card.Content class="flex flex-col gap-4">
    {#each speakers as id}
      <div class="speaker-item flex gap-2">
        <label>
          <Input type="text" class="max-w-xs" bind:value={speakerNames[id]} />
        </label>
        <Button onclick={() => playSpeaker(id)} variant="outline" size="icon">
          {#if currentlyPlayingSpeaker === id}
            <span class="sr-only">Pause</span>
            <Pause />
          {:else}
            <span class="sr-only">Play</span>
            <Play />
          {/if}
        </Button>
      </div>
    {/each}
  </Card.Content>
  <Card.Footer>
    <Button onclick={saveNames} disabled={saving} class="save-button">
      {saving ? "Saving..." : "Save Names"}
    </Button>
  </Card.Footer>
  <audio
    bind:this={audioElem}
    src={audioURL}
    ontimeupdate={onTimeUpdate}
    preload="metadata"
  ></audio>
</Card.Root>
