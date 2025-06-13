<script lang="ts">
  // export let data: { recordings: string[] };
  const { data } = $props();
  import * as Card from "$lib/components/ui/card/index.js";
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

  import FileDrop from "svelte-tauri-filedrop";
  import { toast, Toaster } from "svelte-sonner";

  async function open(path: string) {
    console.log("Files dropped:", path);

    try {
      await invoke("convert_user_audio", { audioPath: path });
      toast.success("File processed successfully!");
      // Refresh the meetings list after processing
      getMeetings();
    } catch (error) {
      console.error("Error processing dropped files:", error);
      toast.error("Error processing files: " + error);
    }
  }

  let meetings: {
    id: string;
    name: string | null;
    created_at: string | null;
  }[] = $state([]);

  onMount(async () => {
    getMeetings();
  });

  async function getMeetings() {
    try {
      const rawMeetings = (await invoke("get_meetings")) as Array<{
        id: string;
        name?: string;
        created_at?: string;
      }>;
      // Sort meetings by date (newest first)
      meetings = rawMeetings
        .map((meeting) => ({
          id: meeting.id,
          name: meeting.name ?? null,
          created_at: meeting.created_at ?? null,
        }))
        .sort((a, b) => {
          const dateA = new Date(a.created_at || "1970-01-01");
          const dateB = new Date(b.created_at || "1970-01-01");
          return dateB.getTime() - dateA.getTime();
        });
      console.log("Meetings fetched and sorted:", meetings);
    } catch (error) {
      console.error("Error fetching meetings:", error);
    }
  }

  const columns: ColumnDef<{ id: string }, any>[] = [
    {
      accessorKey: "name",
      header: "Meeting Name",
      cell: (info) => info.getValue() || `Meeting ${info.row.original.id}`,
    },
    {
      accessorKey: "created_at",
      header: "Date",
      cell: (info) => {
        const date = info.getValue();
        if (!date) return "Unknown";
        return new Date(date).toLocaleDateString("en-US", {
          year: "numeric",
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        });
      },
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

<div class="flex gap-8 p-8 flex-col overflow-y-scroll h-full">
  <Toaster />
  <div class="flex flex-col items-start gap-2">
    <div class="flex items-center justify-between w-full">
      <h1
        class="text-2xl font-bold leading-tight tracking-tighter sm:text-3xl md:text-4xl lg:leading-[1.1]"
      >
        Meeting Summariser
      </h1>
      <Button size="icon" variant="outline" href="/settings">
        <span class="sr-only">Settings</span>
        <Settings />
      </Button>
    </div>

    <p
      class="text-foreground max-w-2xl text-balance text-base font-light sm:text-lg"
    >
      Record a meeting and get a summarisation with key points and action items.
      All local, no cloud processing.
    </p>

    <div class="flex w-full items-center justify-start gap-2 pt-2">
      <Button href="/meeting">New Recording</Button>
    </div>
  </div>

  <Card.Root>
    <Card.Header>
      <Card.Title>Meetings</Card.Title>
    </Card.Header>
    <Card.Content class="">
      <div class="rounded-md border">
        <Table.Root class="w-full">
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
    </Card.Content>
  </Card.Root>
  <FileDrop
    extensions={["mp3", "wav", "ogg", "m4a"]}
    handleOneFile={open}
    let:files
  >
    <div class="dropzone" class:droppable={files.length > 0}>
      <h2>Drop Audio files</h2>
    </div>
  </FileDrop>
</div>

<style>
  .dropzone {
    margin: 20px;
    padding: 20px;
    background: #ffffff08;
  }
  .droppable {
    background: #d6dff0;
  }
</style>
