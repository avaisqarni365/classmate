<script lang="ts">

  import { onMount } from "svelte";

  import { api } from "$lib/api";

  import type { User } from "$lib/types";



  let users = $state<User[]>([]);

  let error = $state("");

  let showForm = $state(false);

  let editingPhoneId = $state<string | null>(null);

  let phoneEdit = $state("");



  let email = $state("");

  let name = $state("");

  let role = $state("student");

  let password = $state("");



  async function load() {

    error = "";

    try {

      users = await api.listUsers();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to load users";

    }

  }



  async function createUser(event: Event) {

    event.preventDefault();

    error = "";

    try {

      await api.createUser({ email, name, role, password });

      email = "";

      name = "";

      role = "student";

      password = "";

      showForm = false;

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to create user";

    }

  }



  function startPhoneEdit(user: User) {

    editingPhoneId = user.id;

    phoneEdit = user.phone ?? "";

  }



  async function savePhone(userId: string) {

    error = "";

    try {

      await api.updateUserPhone(userId, phoneEdit.trim() || null);

      editingPhoneId = null;

      phoneEdit = "";

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to update phone";

    }

  }

  async function exportWhatsAppGdpr(user: User) {
    error = "";
    try {
      const data = await api.exportWhatsAppGdpr(user.id);
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `whatsapp-gdpr-${user.email.replace(/@.*/, "")}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = e instanceof Error ? e.message : "Export failed";
    }
  }



  onMount(load);

</script>



<div class="page-header">

  <div>

    <h2>Users</h2>

    <p>Manage teachers, students, admins, and parents. Add phone numbers for WhatsApp sharing.</p>

  </div>

  <button class="btn btn-primary" onclick={() => (showForm = !showForm)}>

    {showForm ? "Cancel" : "Add user"}

  </button>

</div>



{#if error}

  <p class="error">{error}</p>

{/if}



{#if showForm}

  <div class="card" style="margin-bottom: 1rem">

    <form class="form-grid two-col" onsubmit={createUser}>

      <label>

        Full name

        <input bind:value={name} required />

      </label>

      <label>

        Email

        <input type="email" bind:value={email} required />

      </label>

      <label>

        Role

        <select bind:value={role}>

          <option value="admin">Admin</option>

          <option value="teacher">Teacher</option>

          <option value="student">Student</option>

          <option value="parent">Parent</option>

        </select>

      </label>

      <label>

        Password

        <input type="password" bind:value={password} minlength="6" required />

      </label>

      <div>

        <button class="btn btn-primary" type="submit">Create user</button>

      </div>

    </form>

  </div>

{/if}



<div class="card table-wrap">

  {#if users.length === 0}

    <p class="empty">No users found.</p>

  {:else}

    <table>

      <thead>

        <tr>

          <th>Name</th>

          <th>Email</th>

          <th>Phone</th>

          <th>Role</th>

          <th>Created</th>

          <th></th>

        </tr>

      </thead>

      <tbody>

        {#each users as user}

          <tr>

            <td>{user.name}</td>

            <td>{user.email}</td>

            <td>

              {#if editingPhoneId === user.id}

                <div style="display:flex;gap:.35rem;align-items:center">

                  <input bind:value={phoneEdit} placeholder="+1 555 123 4567" />

                  <button class="btn btn-primary btn-sm" type="button" onclick={() => savePhone(user.id)}>Save</button>

                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => (editingPhoneId = null)}>Cancel</button>

                </div>

              {:else}

                {user.phone || "—"}

                <button class="btn btn-secondary btn-sm" type="button" style="margin-left:.35rem" onclick={() => startPhoneEdit(user)}>

                  Edit

                </button>

              {/if}

            </td>

            <td><span class="badge">{user.role}</span></td>

            <td>{new Date(user.created_at).toLocaleDateString()}</td>

            <td>
              {#if user.phone}
                <button class="btn btn-secondary btn-sm" type="button" onclick={() => exportWhatsAppGdpr(user)}>
                  WhatsApp GDPR
                </button>
              {/if}
            </td>

          </tr>

        {/each}

      </tbody>

    </table>

  {/if}

</div>


