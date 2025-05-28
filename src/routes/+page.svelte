<script lang="ts">
	// export let data: { recordings: string[] };
	const { data } = $props();
	import Recorder from "$lib/components/Recorder.svelte";
	import { Button } from "$lib/components/ui/button/index.js";
	import { createSvelteTable } from "$lib/components/ui/data-table/data-table.svelte.js"
	import { renderComponent } from "$lib/components/ui/data-table/render-helpers.js"
	import { type ColumnDef, getCoreRowModel } from "@tanstack/table-core"
		import * as Table from '$lib/components/ui/table/index.js';


	// When using the Tauri API npm package:
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from "svelte"
	import { FlexRender } from "$lib/components/ui/data-table/index.js"

	let meetings: string[] = $state([]);

	onMount(async () => {
		getMeetings();
	});

	async function getMeetings() {
		try {
			meetings = await invoke("get_meetings");
		} catch (error) {
			console.error("Error fetching meetings:", error);
		}
	}

	async function addMeeting() {
		const newMeeting = await invoke("add_meeting", { name: "New Meeting" });
		getMeetings();
	}

	const columns: ColumnDef<{ id: string }, any>[] = [
		{
			accessorKey: 'id',
			header: 'Recording ID',
			cell: (info) => info.getValue()
		},
		// {
		// 	id: 'actions',
		// 	cell: ({ row }) => {
		// 		// You can pass whatever you need from `row.original` to the component
		// 		return renderComponent(DataTableActions, { id: row.original.id });
		// 	}
		// }
	];

	const table = createSvelteTable({
		get data() {
			return meetings.map((id) => ({ id }));
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});
</script>

<div class="contianer p-4">
	<h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
		Transcriber
	</h1>
	<p>Record a meeting and get the meeting notes!</p>
	<Recorder />

	<a href="/record">New Recording</a>

	<Button onclick={addMeeting}>Add Meeting</Button>

	<p>Meetings:</p>

<Table.Root>
			<Table.Header>
				{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<Table.Row>
						{#each headerGroup.headers as header (header.id)}
							<Table.Head>
								{#if !header.isPlaceholder}
									<FlexRender
										content={header.column.columnDef.header}
										context={header.getContext()}
									/>
								{/if}
							</Table.Head>
						{/each}
					</Table.Row>
				{/each}
			</Table.Header>
			<Table.Body>
				{#each table.getRowModel().rows as row (row.id)}
					<Table.Row data-state={row.getIsSelected() && 'selected'}>
						{#each row.getVisibleCells() as cell (cell.id)}
							<Table.Cell>
								<a href={`/record/${cell.column.columnDef.cell.id}`}>
									<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
								</a>
							</Table.Cell>
						{/each}
					</Table.Row>
				{:else}
					<Table.Row>
						<Table.Cell colspan={columns.length} class="h-24 text-center">No results.</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
</div>
