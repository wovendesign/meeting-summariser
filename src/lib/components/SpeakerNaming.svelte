<script lang="ts">
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import Play from '@lucide/svelte/icons/play';
	import Pause from '@lucide/svelte/icons/pause';
	import { Input } from '$lib/components/ui/input';
	import { onMount } from 'svelte';

	const {
		audioURL,
		jsonPath
	}: {
		audioURL: string;
		jsonPath: string;
	} = $props();
	interface Segment {
		start: number;
		end: number;
		speaker: string;
	}

	let segments: Segment[] = [];
	let speakers: string[] = $state([]);
	let speakerNames: Record<string, string> = {};
	let audioElem!: HTMLAudioElement;
	let playQueue: Segment[] = [];
	let queueIndex = 0;
	let saving = $state(false);
	let saveStatus = '';
	let currentlyPlayingSpeaker: string | null = $state(null);

	onMount(async () => {
		const res = await fetch(`/api/segments?filepath=${encodeURIComponent(jsonPath)}`);
		const data = await res.json();
		segments = data.segments;
		// collect unique speaker IDs
		const set = new Set<string>(segments.map((s: Segment) => s.speaker));
		speakers = Array.from(set);
		speakers.forEach((id) => (speakerNames[id] = id));
	});

	function playSpeaker(id: string) {
		if (currentlyPlayingSpeaker === id) {
			audioElem.pause();
			currentlyPlayingSpeaker = null;
			return;
		}
		playQueue = segments.filter((s) => s.speaker === id).sort((a, b) => a.start - b.start);
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
		saving = true;
		saveStatus = '';
		try {
			const res = await fetch('/api/segments/update', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ jsonPath, speakerNames })
			});
			saveStatus = res.ok ? 'Speaker names saved' : `Save failed: ${res.statusText}`;
		} catch (err) {
			saveStatus = `Error: ${err instanceof Error ? err.message : err}`;
		} finally {
			saving = false;
		}
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
			{saving ? 'Saving...' : 'Save Names'}
		</Button>
	</Card.Footer>
	<audio bind:this={audioElem} src={audioURL} ontimeupdate={onTimeUpdate} preload="metadata"
	></audio>
</Card.Root>
