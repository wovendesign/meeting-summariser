<script lang="ts" >
	import SpeakerNaming from '$lib/components/SpeakerNaming.svelte';
	import Clipboard from '@lucide/svelte/icons/clipboard';

	// const {
	// 	id,
	// 	jsonPath,
	// 	txtPath,
	// 	webmUrl,
	// 	transcript,
	// 	summary
	// }: {
	// 	id: string;
	// 	jsonPath: string;
	// 	txtPath: string;
	// 	webmUrl: string;
	// 	transcript: string;
	// 	summary: string;
	// } = $props();

	let { data }: PageProps = $props();


	import { marked } from 'marked';
	import type { PageProps } from './$types';
	import * as Card from '$lib/components/ui/card/index.js';
	import Button from '$lib/components/ui/button/button.svelte';
	import { Textarea } from '$lib/components/ui/textarea';
	import { onMount } from 'svelte'
	let transcriptContent = $state("");
	let summaryContent = $state("");
	let savingTranscript = $state(false);
	let saveStatus = $state('');
	let loadingSummary = $state(false);

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

	})
</script>

<div class="container flex flex-col gap-4 p-4">
	<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
		Recording {data.id}
	</h1>
	<!-- <audio src={data.webmUrl} controls></audio> -->

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
		<!-- <SpeakerNaming audioURL={data.webmUrl} jsonPath={data.jsonPath} /> -->
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
