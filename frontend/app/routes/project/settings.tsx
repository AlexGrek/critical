import { type LoaderFunction, type ActionFunction } from "react-router";
import { useLoaderData, Form } from "react-router";

export const loader: LoaderFunction = async ({ params }) => {
  const settings = await fetchProjectSettings(params.projectId);
  return { settings };
};

export const action: ActionFunction = async ({ request, params }) => {
  const formData = await request.formData();
  await updateProjectSettings(params.projectId, formData);
  return { success: true };
};

export default function ProjectSettings() {
  const { settings } = useLoaderData<typeof loader>();

  return (
    <div>
      <h2>Project Settings</h2>
      <Form method="post">
        <div>
          <label>
            Project Name:
            <input 
              name="name" 
              defaultValue={settings.name}
              required 
            />
          </label>
        </div>
        <div>
          <label>
            Description:
            <textarea 
              name="description" 
              defaultValue={settings.description}
            />
          </label>
        </div>
        <button type="submit">Save Settings</button>
      </Form>
    </div>
  );
}