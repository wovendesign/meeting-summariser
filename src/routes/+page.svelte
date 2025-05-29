<script lang="ts">
  // export let data: { recordings: string[] };
  const { data } = $props();
  import Recorder from "$lib/components/Recorder.svelte";
  import Settings from "@lucide/svelte/icons/settings";
  import { Button } from "$lib/components/ui/button/index.js";
  import { createSvelteTable } from "$lib/components/ui/data-table/data-table.svelte.js";
  import { renderComponent } from "$lib/components/ui/data-table/render-helpers.js";
  import { type ColumnDef, getCoreRowModel } from "@tanstack/table-core";
  import * as Table from "$lib/components/ui/table/index.js";

  // When using the Tauri API npm package:
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { FlexRender } from "$lib/components/ui/data-table/index.js";

  let meetings: {
    id: string;
    name: string | null;
    // Add other fields as necessary
  }[] = $state([]);

  onMount(async () => {
    getMeetings();
  });

  async function getMeetings() {
    try {
      meetings = await invoke("get_meetings");
      console.log("Meetings fetched:", meetings);
    } catch (error) {
      console.error("Error fetching meetings:", error);
    }
  }

  const columns: ColumnDef<{ id: string }, any>[] = [
    {
      accessorKey: "name",
      header: "Meeting Name",
      cell: (info) => info.getValue(),
    },
    {
      accessorKey: "id",
      header: "Recording ID",
      cell: (info) => info.getValue(),
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
      return meetings;
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
  });
</script>

<div class="flex gap-2 p-4 flex-col">
  <div class="flex items-center justify-between">
    <h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
      Transcriber
    </h1>
    <Button size="icon" variant="outline" href="/settings">
      <span class="sr-only">Settings</span>
      <Settings />
    </Button>
  </div>
  <p>Record a meeting and get the meeting notes!</p>

  <Button href="/meeting">New Recording</Button>

  <div class="rounded-md border">
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
          <Table.Row data-state={row.getIsSelected() && "selected"}>
            {#each row.getVisibleCells() as cell (cell.id)}
              <Table.Cell>
                <a href={`/meeting/${row.getValue("id")}`}>
                  <FlexRender
                    content={cell.column.columnDef.cell}
                    context={cell.getContext()}
                  />
                </a>
              </Table.Cell>
            {/each}
          </Table.Row>
        {:else}
          <Table.Row>
            <Table.Cell colspan={columns.length} class="h-24 text-center"
              >No results.</Table.Cell
            >
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </div>
</div>
