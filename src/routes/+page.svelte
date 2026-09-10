<script lang="ts">
  import "../app.css";
  import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
  import { REGEXP_ONLY_DIGITS } from "bits-ui";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as InputOTP from "$lib/components/ui/input-otp/index.js";
  import * as Table from "$lib/components/ui/table/index.js";

  import { invoke } from "@tauri-apps/api/core";
  import { load } from '@tauri-apps/plugin-store';
  import {
    startService,
    stopService,
    isServiceRunning,
    getLifecycleStatus,
    onPluginEvent,
    getPlatformCapabilities,
    configureRecovery,
    validateBackgroundServiceSetup,
    normalizeBackgroundServiceError,
  } from 'tauri-plugin-background-service';

  interface RawDevice {
    device_index: number,
    device_name: string,
    device_id: string | null,
    device_requires_pin: boolean,
  }

  interface Device extends RawDevice {
    device_pin_entry_locked: boolean,
    device_pin: string,
    device_already_known: boolean
  }
  
  interface DeviceListResponse {
    DeviceList: RawDevice[]
  }
  
  interface DeviceResponse {
    Device: Device
  }
  
  interface ErrorResponse {
    Error: string
  }

  type ClientResponse = DeviceListResponse | DeviceResponse | ErrorResponse

  const pin_length = 4

  const persisted_identity_store = await load('persisted_identities.json', { autoSave: false });

  var device_list: Device[] = $state([])

  var connected_device: Device | null = $state(null)

  var err_msg: string | null = $state(null)

  function pinDialogStateChange(index:number) {
    device_list[index].device_pin_entry_locked = true
    device_list[index].device_pin = ""
  }

  async function getclients() {
    try {
      await invoke<ClientResponse>('get_client_list', { timeout: 5 }).then(async (reply) => {
        if ("DeviceList" in reply) {
          const device_list_promises = reply.DeviceList.map(async (DeviceObject) => {
            const already_known = DeviceObject.device_id? await persisted_identity_store.has(DeviceObject.device_id) : false
            return {
              device_index: DeviceObject.device_index,
              device_name: DeviceObject.device_name,
              device_id: DeviceObject.device_id,
              device_requires_pin: DeviceObject.device_requires_pin,
              device_pin_entry_locked: true,
              device_pin: "",
              device_already_known: already_known
            }
          })
          device_list = await Promise.all(device_list_promises)
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function connectClientIndex(index: number) {
    try {
      await invoke<ClientResponse>('connect_to_client_index', { index: index }).then((reply) => {
        if ("Device" in reply) {
          connected_device = device_list[reply.Device.device_index]
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function connectClientIndexPersistedID(index: number) {
    try {
      await invoke<ClientResponse>('connect_to_client_index_persisted_id', { index: index }).then(async (reply) => {
        if ("Device" in reply) {
          connected_device = device_list[reply.Device.device_index]
          connected_device.device_already_known = connected_device.device_id ? await persisted_identity_store.has(connected_device.device_id) : false
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function connectClientIndexPin(index: number, pin: string) {
    try {
      await invoke<ClientResponse>('connect_to_client_index_pin_pairing', { index: index, pin: pin }).then((reply) => {
        if ("Device" in reply) {
          connected_device = device_list[reply.Device.device_index]
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
  }

  async function requestPINIndex(index: number) {
    try {
      await invoke<ClientResponse>('send_client_index_pin_prompt', { index: index }).then((reply) => {
        if ("HttpResponse" in reply) {
          device_list[index].device_pin_entry_locked = false
          console.log(reply)
          err_msg = null
        }
      })
    }
    catch (error) {
      err_msg = JSON.stringify(error as Object)
    }
	}
  /*
  // Query platform capabilities (call early to set UI expectations)
  const caps = await getPlatformCapabilities();
  console.log(caps.backgroundExecution);  // 'guaranteed' | 'bestEffort' | 'unsupported'
  console.log(caps.survivesReboot);       // 'guaranteed' | 'bestEffort' | 'unsupported'

  // Start the service (optionally configure the Android notification label)
  await startService({ serviceLabel: 'Casting via Airplay', foregroundServiceType: "mediaProjection" });

  // Check if running (simple boolean)
  const running = await isServiceRunning();

  // Query detailed service state
  const status = await getLifecycleStatus();
  console.log(status.state); // 'idle' | 'initializing' | 'running' | 'stopped'
  console.log(status.lastError); // null or error message
  console.log(status.desiredRunning); // true | false | undefined

  // Listen to lifecycle events
  const unlisten = await onPluginEvent((event) => {
    switch (event.type) {
      case 'started':
        console.log('Service started');
        break;
      case 'stopped':
        console.log('Service stopped:', event.reason);
        break;
      case 'error':
        console.error('Service error:', event.message);
        break;
    }
  });

  // Validate your platform setup (checks permissions, manifest entries)
  const report = await validateBackgroundServiceSetup();
  if (!report.ok) {
    for (const err of report.errors) {
      console.error(`[${err.code}] ${err.message}`);
      if (err.fix) console.error(`  Fix: ${err.fix}`);
    }
  }

  // Stop the service
  await stopService();

  // Clean up listener
  unlisten();*/

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<div id="app" class="relative min-h-dvh">
    <h1>Connect to Airplay</h1>
    <div class="flex flex-row w-full gap-2 justify-center my-5">
      <Button type="button" onclick={getclients}>Refresh client list</Button>
    </div>
    <Table.Root>
      <Table.Body>
      
        {#each device_list as device}
          <Table.Row>
            <Table.Cell>{device.device_index}</Table.Cell>
            <Table.Cell>{device.device_name}</Table.Cell>
            <Table.Cell>{device.device_id}</Table.Cell>
            <Table.Cell>{device.device_requires_pin}</Table.Cell>
              {#if device.device_requires_pin}
                <Table.Cell>
                  <Button disabled={!device.device_already_known} type="button" variant="outline" onclick={() => connectClientIndexPersistedID(device.device_index)}>Connect</Button>
                </Table.Cell>
                <Table.Cell>
                  <Dialog.Root onOpenChange={() => pinDialogStateChange(device.device_index)}>
                      <Dialog.Trigger type="button" class={buttonVariants({ variant: "outline" })}>Pair</Dialog.Trigger>
                      <Dialog.Content class="sm:max-w-[425px]">
                        <Dialog.Header />
                        <div class="flex flex-col gap-5 items-center">
                          <Button class="w-fit" type="button" onclick={() => requestPINIndex(device.device_index)} variant="outline">Request PIN</Button>
                          <InputOTP.Root bind:value={device.device_pin} disabled={device.device_pin_entry_locked} maxlength={pin_length} pattern={REGEXP_ONLY_DIGITS} textalign="center" class="w-fit">
                            {#snippet children({ cells })}
                              <InputOTP.Group>
                                {#each cells as cell (cell)}
                                  <InputOTP.Slot {cell} />
                                {/each}
                              </InputOTP.Group>
                            {/snippet}
                          </InputOTP.Root>
                          <Button disabled={device.device_pin.length < pin_length} class="w-fit" type="button" variant="outline" onclick={() => connectClientIndexPin(device.device_index,device.device_pin)}>Connect</Button>
                        </div>
                      </Dialog.Content>
                  </Dialog.Root>
                </Table.Cell>
              {:else}
                <Table.Cell />
                <Table.Cell>
                  <Button type="button" onclick={() => connectClientIndex(device.device_index)} variant="outline">Connect</Button>
                </Table.Cell>
              {/if}
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
    
    {#if err_msg}
      Error: {err_msg}
    {/if}
</div>