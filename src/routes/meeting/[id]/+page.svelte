<script lang="ts" >
	import SpeakerNaming from '$lib/components/SpeakerNaming.svelte';
	import Clipboard from '@lucide/svelte/icons/clipboard';

	let { data }: PageProps = $props();


	import { marked } from 'marked';
	import type { PageProps } from './$types';
	import * as Card from '$lib/components/ui/card/index.js';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Textarea } from '$lib/components/ui/textarea';
	import { onMount } from 'svelte'
	import { invoke } from '@tauri-apps/api/core'

	let transcriptContent = $state("");
	let transcriptJsonContent: string | null = $state(null);
	let summaryContent: string | null = $state("");
	let savingTranscript = $state(false);
	let saveStatus = $state('');
	let loadingSummary = $state(false);

	let audio: ArrayBuffer | null = $state(null);
	let audioURL = $derived.by(() => {
		if (!audio) return '';
		const blob = new Blob([audio], { type: 'audio/wav' });
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
		// loadingSummary = true;
		// try {
		// 	const res = await fetch(`/api/summarize?filepath=${encodeURIComponent(data.txtPath)}`);
		// 	const json = await res.json();
		// 	summaryContent = json.summary;
		// } catch (err) {
		// 	console.error('Error regenerating summary:', err);
		// } finally {
		// 	loadingSummary = false;
		// }
	}

	const markdownContent = $derived.by(() => {
		if (!summaryContent) return '';
		const realNewLinesContent = summaryContent.replace(/\n/g, '\n');
		return marked(realNewLinesContent, {
			gfm: true,
			breaks: true
		});
	});

	onMount(async () => {
		await getTranscript();
		await getSummary();
		await getAudio();
		await getTranscriptJson();
	})

	async function getTranscript() {
		try {
			transcriptContent = await invoke("get_meeting_transcript", { meetingId: data.id })
		} catch (error) {
			console.error("Error fetching transcript:", error);
			transcriptContent = "Error fetching transcript";
		}
	}

	async function getTranscriptJson() {
		try {
			transcriptJsonContent = await invoke("get_meeting_transcript_json", { meetingId: data.id })
		} catch (error) {
			console.error("Error fetching transcript:", error);
			transcriptJsonContent = null;
		}
	}

	async function getSummary() {
		try {
			summaryContent = await invoke("get_meeting_summary", { meetingId: data.id })
		} catch (error) {
			console.error("Error fetching summary:", error);
			summaryContent = null;
		}
	}

	async function getAudio() {
		try {
			audio = await invoke("get_meeting_audio", { meetingId: data.id });
		} catch (error) {
			console.error("Error fetching audio:", error);
			audio = null;
		}
	}
</script>

<div class="container flex flex-col gap-4 p-4">
	<Button variant="ghost" href="/" class="self-start">Back</Button>
	<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
		Recording {data.id}
	</h1>
	<audio src={audioURL} controls></audio>

	<section>
		{#if saveStatus}
			<p>{saveStatus}</p>
		{/if}

		<Card.Root>
			<Card.Header>
				<Card.Title>Transcript</Card.Title>
			</Card.Header>
			<Card.Content class="prose prose-invert">
				<Textarea bind:value={transcriptContent} placeholder="Edit the Transcript" />
			</Card.Content>
			<Card.Footer class="flex gap-2">
				<Button onclick={saveTranscript} disabled={savingTranscript}>
					{savingTranscript ? 'Saving...' : 'Save Transcript'}
				</Button>
			</Card.Footer>
		</Card.Root>
	</section>

	<section>
		<SpeakerNaming audioURL={audioURL} json={transcriptJsonContent} />
	</section>

	<section>
		<Card.Root>
			<Card.Header>
				<Card.Title>Transcription Summary</Card.Title>
			</Card.Header>
			<Card.Content class="prose prose-invert">
				{#if summaryContent}
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
					{loadingSummary ? 'Regenerating...' : 'Regenerate Summary'}
				</Button>
			</Card.Footer>
		</Card.Root>
	</section>
</div>
