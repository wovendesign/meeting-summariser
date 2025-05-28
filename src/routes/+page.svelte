<script lang="ts">
	// export let data: { recordings: string[] };
	const { data } = $props();
	import Recorder from "$lib/components/Recorder.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
</script>

<div class="contianer p-4">
	<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
		Transcriber
	</h1>
	<p>Record a meeting and get the meeting notes!</p>
	<Recorder />

	<a href="/record">New Recording</a>

	<Button>Test</Button>
</div>
