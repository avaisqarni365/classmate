<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n";
  import type {
    CashbookEntry,
    CashbookSettings,
    CashbookSummary,
    Course,
    User,
  } from "$lib/types";

  let settings = $state<CashbookSettings | null>(null);
  let summary = $state<CashbookSummary | null>(null);
  let entries = $state<CashbookEntry[]>([]);
  let users = $state<User[]>([]);
  let courses = $state<Course[]>([]);
  let fromDate = $state("");
  let toDate = $state("");
  let error = $state("");
  let message = $state("");
  let integrationResult = $state("");

  let direction = $state("income");
  let category = $state("student_fee");
  let amount = $state("");
  let paymentMethod = $state("cash");
  let userId = $state("");
  let courseId = $state("");
  let description = $state("");
  let reference = $state("");
  let entryDate = $state("");

  let currency = $state("USD");
  let invoiceNinjaUrl = $state("");
  let invoiceNinjaToken = $state("");

  const categoryOptions = $derived(
    direction === "income"
      ? [
          { value: "student_fee", label: t("fees.categoryStudentFee") },
          { value: "other_income", label: t("fees.categoryOtherIncome") },
        ]
      : [
          { value: "teacher_salary", label: t("fees.categoryTeacherSalary") },
          { value: "other_expense", label: t("fees.categoryOtherExpense") },
        ],
  );

  $effect(() => {
    if (direction === "income" && !["student_fee", "other_income"].includes(category)) {
      category = "student_fee";
    }
    if (direction === "expense" && !["teacher_salary", "other_expense"].includes(category)) {
      category = "teacher_salary";
    }
  });

  async function load() {
    error = "";
    try {
      settings = await api.getCashbookSettings();
      currency = settings.currency;
      invoiceNinjaUrl = settings.invoice_ninja_url;
      summary = await api.getCashbookSummary(fromDate || undefined, toDate || undefined);
      entries = await api.listCashbookEntries(fromDate || undefined, toDate || undefined);
      users = await api.listUsers();
      courses = await api.listCourses();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load cash book";
    }
  }

  async function saveSettings() {
    try {
      settings = await api.saveCashbookSettings({
        currency: currency.trim() || "USD",
        invoice_ninja_url: invoiceNinjaUrl.trim() || undefined,
        invoice_ninja_token: invoiceNinjaToken.trim() || undefined,
      });
      invoiceNinjaToken = "";
      message = t("fees.settingsSaved");
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save settings";
    }
  }

  async function testIntegration() {
    integrationResult = "";
    try {
      const result = await api.testInvoiceNinjaConnection();
      integrationResult = result.message;
    } catch (e) {
      integrationResult = e instanceof Error ? e.message : "Connection failed";
    }
  }

  async function addEntry(event: Event) {
    event.preventDefault();
    const parsed = Number(amount);
    if (!parsed || parsed <= 0) {
      error = t("fees.invalidAmount");
      return;
    }
    try {
      await api.createCashbookEntry({
        direction,
        category,
        amount: parsed,
        currency,
        description: description.trim() || undefined,
        user_id: userId || undefined,
        course_id: courseId || undefined,
        payment_method: paymentMethod,
        reference: reference.trim() || undefined,
        entry_date: entryDate || undefined,
      });
      amount = "";
      description = "";
      reference = "";
      userId = "";
      courseId = "";
      message = t("fees.entrySaved");
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save entry";
    }
  }

  async function removeEntry(id: string) {
    if (!confirm(t("fees.deleteConfirm"))) return;
    try {
      await api.deleteCashbookEntry(id);
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete entry";
    }
  }

  async function exportCsv() {
    try {
      const csv = await api.exportCashbookCsv(fromDate || undefined, toDate || undefined);
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `cashbook-${new Date().toISOString().slice(0, 10)}.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to export";
    }
  }

  function formatMoney(value: number, cur: string) {
    return new Intl.NumberFormat(undefined, { style: "currency", currency: cur }).format(value);
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>{t("fees.title")}</h2>
    <p>{t("fees.subtitle")}</p>
  </div>
</div>

{#if error}<p class="error">{error}</p>{/if}
{#if message}<p style="color:var(--success);margin-bottom:1rem">{message}</p>{/if}

{#if summary}
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(10rem,1fr));gap:.75rem;margin-bottom:1rem">
    <div class="card">
      <small style="color:var(--muted)">{t("fees.totalIncome")}</small>
      <strong>{formatMoney(summary.total_income, summary.currency)}</strong>
    </div>
    <div class="card">
      <small style="color:var(--muted)">{t("fees.totalExpense")}</small>
      <strong>{formatMoney(summary.total_expense, summary.currency)}</strong>
    </div>
    <div class="card">
      <small style="color:var(--muted)">{t("fees.balance")}</small>
      <strong>{formatMoney(summary.balance, summary.currency)}</strong>
    </div>
    <div class="card">
      <small style="color:var(--muted)">{t("fees.entries")}</small>
      <strong>{summary.entry_count}</strong>
    </div>
  </div>
{/if}

<div class="card" style="margin-bottom:1rem">
  <h3 style="margin-top:0">{t("fees.filterTitle")}</h3>
  <div class="form-grid three-col">
    <label>{t("fees.fromDate")}<input type="date" bind:value={fromDate} /></label>
    <label>{t("fees.toDate")}<input type="date" bind:value={toDate} /></label>
    <div style="align-self:end;display:flex;gap:.5rem;flex-wrap:wrap">
      <button class="btn btn-secondary" type="button" onclick={load}>{t("fees.applyFilter")}</button>
      <button class="btn btn-secondary" type="button" onclick={exportCsv}>{t("fees.exportCsv")}</button>
    </div>
  </div>
</div>

<div class="card" style="margin-bottom:1rem">
  <h3 style="margin-top:0">{t("fees.addEntry")}</h3>
  <form class="form-grid two-col" onsubmit={addEntry}>
    <label>
      {t("fees.direction")}
      <select bind:value={direction}>
        <option value="income">{t("fees.income")}</option>
        <option value="expense">{t("fees.expense")}</option>
      </select>
    </label>
    <label>
      {t("fees.category")}
      <select bind:value={category}>
        {#each categoryOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>
    <label>{t("fees.amount")}<input bind:value={amount} type="number" min="0.01" step="0.01" required /></label>
    <label>
      {t("fees.paymentMethod")}
      <select bind:value={paymentMethod}>
        <option value="cash">{t("fees.methodCash")}</option>
        <option value="bank">{t("fees.methodBank")}</option>
        <option value="cheque">{t("fees.methodCheque")}</option>
        <option value="online">{t("fees.methodOnline")}</option>
      </select>
    </label>
    <label>
      {t("fees.linkedUser")}
      <select bind:value={userId}>
        <option value="">{t("fees.none")}</option>
        {#each users as u}
          <option value={u.id}>{u.name} ({u.role})</option>
        {/each}
      </select>
    </label>
    <label>
      {t("fees.linkedCourse")}
      <select bind:value={courseId}>
        <option value="">{t("fees.none")}</option>
        {#each courses as c}
          <option value={c.id}>{c.code} — {c.title}</option>
        {/each}
      </select>
    </label>
    <label>{t("fees.entryDate")}<input type="date" bind:value={entryDate} /></label>
    <label>{t("fees.reference")}<input bind:value={reference} placeholder="TXN-001" /></label>
    <label style="grid-column:1/-1">{t("fees.description")}<textarea bind:value={description} rows="2"></textarea></label>
    <button class="btn btn-primary" type="submit">{t("fees.saveEntry")}</button>
  </form>
</div>

<div class="card" style="margin-bottom:1rem">
  <h3 style="margin-top:0">{t("fees.ledgerTitle")}</h3>
  {#if entries.length === 0}
    <p class="empty">{t("fees.noEntries")}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("fees.entryDate")}</th>
          <th>{t("fees.direction")}</th>
          <th>{t("fees.category")}</th>
          <th>{t("fees.amount")}</th>
          <th>{t("fees.linkedUser")}</th>
          <th>{t("fees.description")}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each entries as entry}
          <tr>
            <td>{entry.entry_date}</td>
            <td>{entry.direction}</td>
            <td>{entry.category}</td>
            <td>{formatMoney(entry.amount, entry.currency)}</td>
            <td>{entry.user_name ?? "—"}</td>
            <td>{entry.description ?? entry.reference ?? "—"}</td>
            <td><button class="btn btn-danger btn-sm" type="button" onclick={() => removeEntry(entry.id)}>{t("fees.delete")}</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <h3 style="margin-top:0">{t("fees.integrationTitle")}</h3>
  <p style="color:var(--muted);font-size:.9rem">{t("fees.integrationHint")}</p>
  <div class="form-grid two-col" style="margin-top:.75rem">
    <label>{t("fees.currency")}<input bind:value={currency} placeholder="USD" /></label>
    <label>{t("fees.invoiceNinjaUrl")}<input bind:value={invoiceNinjaUrl} placeholder="https://invoicing.example.com" /></label>
    <label style="grid-column:1/-1">
      {t("fees.invoiceNinjaToken")}
      <input bind:value={invoiceNinjaToken} type="password" placeholder={settings?.invoice_ninja_configured ? "••••••••" : "API token"} />
    </label>
  </div>
  <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">
    <button class="btn btn-primary" type="button" onclick={saveSettings}>{t("fees.saveSettings")}</button>
    <button class="btn btn-secondary" type="button" onclick={testIntegration} disabled={!settings?.invoice_ninja_configured && !invoiceNinjaUrl.trim()}>
      {t("fees.testIntegration")}
    </button>
  </div>
  {#if integrationResult}
    <p style="margin-top:.75rem;font-size:.9rem;color:var(--muted)">{integrationResult}</p>
  {/if}
</div>
